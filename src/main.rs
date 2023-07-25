mod error;

use clap::Parser;
use std::ffi::OsStr;
use std::iter::Peekable;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::error::Error;

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    #[arg(short = 'd', long = "days")]
    /// Number of days before expiry to start warning
    warn_days: i64,

    #[arg(name = "keys")]
    /// GPG key IDs in long format (40 uppercase hex characters, no spaces)
    keys: Vec<KeyId>,

    #[arg(long)]
    /// Execute gpg --quick-set-expire to set expiration time to <expire> for each key that expires soon
    expire: Option<String>,
}

#[derive(Debug)]
struct KeyIdError<'a>(&'a str);
impl<'a> std::error::Error for KeyIdError<'a> {}
impl<'a> std::fmt::Display for KeyIdError<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct KeyId(String);
impl std::fmt::Display for KeyId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(&self.0)
    }
}
impl std::str::FromStr for KeyId {
    type Err = KeyIdError<'static>;
    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id.len() == 40 && id.chars().all(|c| "0123456789ABCDEF".contains(c)) {
            Ok(KeyId(id.to_string()))
        } else {
            Err(KeyIdError(
                "Key ID must be exactly 40 uppercase hex characters.",
            ))
        }
    }
}
impl AsRef<str> for KeyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl AsRef<OsStr> for KeyId {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

#[derive(Debug)]
struct GpgStatus {
    main_keys: Vec<GpgMainKeyStatus>,
}
impl GpgStatus {
    fn parse_from<'a, I>(it: I) -> Self
    where
        I: Iterator<Item = Vec<&'a str>>,
    {
        let mut main_keys = Vec::new();
        let mut it = it
            .filter(|line| ["pub", "sub", "fpr"].contains(&line[0]))
            .peekable();
        while it.peek().is_some() {
            main_keys.push(GpgMainKeyStatus::parse_from(&mut it));
        }
        Self { main_keys }
    }
}

#[derive(Debug)]
struct GpgMainKeyStatus {
    status: GpgKeyStatus,
    subkeys: Vec<GpgKeyStatus>,
}
impl GpgMainKeyStatus {
    fn parse_from<'a, I>(it: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = Vec<&'a str>>,
    {
        assert_eq!(
            it.peek().expect("Failed to parse GPG key structure")[0],
            "pub",
            "Failed to parse GPG key structure"
        );
        let expires = it.next().expect("Failed to parse GPG key structure")[6]
            .parse()
            .ok();
        let fingerprint = it.next().expect("Failed to parse GPG key structure")[9]
            .parse()
            .expect("Failed to parse main key ID from GPG");
        let mut subkeys = Vec::new();
        while it.peek().map(|line| line[0] != "pub").unwrap_or(false) {
            subkeys.push(GpgKeyStatus::parse_from(it));
        }
        Self {
            status: GpgKeyStatus {
                expires,
                fingerprint,
            },
            subkeys,
        }
    }
}

#[derive(Debug)]
struct GpgKeyStatus {
    fingerprint: KeyId,
    expires: Option<u64>,
}
impl GpgKeyStatus {
    fn expire_days(&self, now_secs: u64) -> Option<i64> {
        self.expires
            .map(|expire| ((expire as i64) - (now_secs as i64)) / (3600 * 24))
    }

    fn parse_from<'a, I>(it: &mut Peekable<I>) -> GpgKeyStatus
    where
        I: Iterator<Item = Vec<&'a str>>,
    {
        let line1 = it.next().expect("Failed to parse GPG key structure");
        let line2 = it.next().expect("Failed to parse GPG key structure");
        GpgKeyStatus {
            fingerprint: line2[9].parse().expect("Failed to parse key ID from GPG"),
            expires: line1[6].parse().ok(),
        }
    }
}

fn run() -> Result<i32, Error> {
    let cli = Cli::parse();

    let args = {
        let mut v: Vec<&str> = vec!["--with-colons", "--fixed-list-mode", "--list-keys"];
        cli.keys.iter().for_each(|k| v.push(k.as_ref()));
        v
    };

    let output = String::from_utf8(
        Command::new("gpg")
            .args(args)
            .output()
            .expect("GPG command failed")
            .stdout,
    )?;
    let status: GpgStatus = GpgStatus::parse_from(
        output
            .lines()
            .map(|line| line.split(':').collect::<Vec<&str>>()),
    );

    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiring_keys: Vec<(&KeyId, i64)> = status
        .main_keys
        .iter()
        .map(|key| &key.status)
        .chain(status.main_keys.iter().flat_map(|key| &key.subkeys))
        .filter(|key_status| cli.keys.contains(&key_status.fingerprint))
        .filter_map(|key_status| match key_status.expire_days(now_secs) {
            Some(days) if days <= cli.warn_days => Some((&key_status.fingerprint, days)),
            _ => None,
        })
        .collect();

    if !expiring_keys.is_empty() {
        println!("The following GPG keys will expire soon:");

        for (fpr, days) in &expiring_keys {
            println!("{fpr}: {days} days");
        }

        if let Some(expire) = cli.expire {
            for GpgMainKeyStatus { status, subkeys } in &status.main_keys {
                if expiring_keys
                    .iter()
                    .any(|(expiring_fpr, _)| **expiring_fpr == status.fingerprint)
                {
                    println!(
                        "Setting expiry to {expire} for main key: {}",
                        status.fingerprint
                    );

                    let update_expiry_cmd = Command::new("gpg")
                        .args(["--quick-set-expire", status.fingerprint.as_ref(), &expire])
                        .spawn()?
                        .wait()?;

                    if !update_expiry_cmd.success() {
                        eprintln!(
                            "Failed to update expiry of main key: {}",
                            status.fingerprint
                        );
                        return Ok(1);
                    }
                }

                let update_subkeys: Vec<&KeyId> = subkeys
                    .iter()
                    .map(|subkey| &subkey.fingerprint)
                    .filter(|fpr| {
                        expiring_keys
                            .iter()
                            .any(|(expiring_fpr, _)| expiring_fpr == fpr)
                    })
                    .collect();

                if !update_subkeys.is_empty() {
                    let update_subkeys_str = update_subkeys
                        .iter()
                        .map(|id| id.as_ref())
                        .collect::<Vec<&str>>()
                        .join(", ");
                    println!(
                        "Setting expiry to {expire} for subkeys: {}",
                        update_subkeys_str
                    );

                    let update_expiry_cmd = Command::new("gpg")
                        .args(["--quick-set-expire", status.fingerprint.as_ref(), &expire])
                        .args(&update_subkeys)
                        .spawn()?
                        .wait()?;

                    if !update_expiry_cmd.success() {
                        eprintln!("Failed to update expiry of subkeys: {}", update_subkeys_str);
                        return Ok(1);
                    }
                }
            }

            Ok(0)
        } else {
            Ok(1)
        }
    } else {
        Ok(0)
    }
}

fn main() {
    std::process::exit(run().unwrap());
}
