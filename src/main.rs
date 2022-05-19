mod error;
mod util;

use error::Error;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use structopt::StructOpt;
use util::GroupedExt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "d", long = "days")]
    /// Number of days before expiry to start warning
    warn_days: i64,

    #[structopt(name = "keys")]
    /// GPG key IDs in long format (40 uppercase hex characters, no spaces)
    keys: Vec<KeyId>,
}

#[derive(Debug)]
struct KeyIdError<'a>(&'a str);
impl<'a> std::error::Error for KeyIdError<'a> {}
impl<'a> std::fmt::Display for KeyIdError<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_str(self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
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
}
impl From<&Vec<Vec<&str>>> for GpgKeyStatus {
    fn from(input: &Vec<Vec<&str>>) -> GpgKeyStatus {
        let line1 = &input[0];
        let line2 = &input[1];
        GpgKeyStatus {
            fingerprint: line2[9].parse().expect("Failed to parse key ID from GPG"),
            expires: line1[6].parse().ok(),
        }
    }
}

fn run() -> Result<i32, Error> {
    let opt = Opt::from_args();

    let args = {
        let mut v: Vec<&str> = vec!["--with-colons", "--fixed-list-mode", "--list-keys"];
        opt.keys.iter().for_each(|k| v.push(k.as_ref()));
        v
    };

    let output = String::from_utf8(
        Command::new("gpg")
            .args(args)
            .output()
            .expect("GPG command failed")
            .stdout,
    )?;
    let keys: Vec<GpgKeyStatus> = output
        .lines()
        .map(|line| line.split(':').collect::<Vec<&str>>())
        .filter(|line| ["pub", "sub", "fpr"].contains(&line[0]))
        .grouped(2)
        .map(|group| (&group).into())
        .collect();

    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiring_keys: Vec<&GpgKeyStatus> = keys
        .iter()
        .filter(|key_status| opt.keys.contains(&key_status.fingerprint))
        .filter(|key_status| match key_status.expire_days(now_secs) {
            Some(days) => days <= opt.warn_days,
            None => false,
        })
        .collect();

    if !expiring_keys.is_empty() {
        println!("The following GPG keys will expire soon:");

        for key_status in &expiring_keys {
            let remaining_days = key_status.expire_days(now_secs).unwrap();
            println!(
                "{fpr}: {days} days",
                fpr = &key_status.fingerprint,
                days = remaining_days
            );
        }

        Ok(1)
    } else {
        Ok(0)
    }
}

fn main() {
    std::process::exit(run().unwrap());
}
