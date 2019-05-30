mod error;
mod util;

use error::Error;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use structopt::StructOpt;
use util::grouped;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "d", long = "days")]
    /// Number of days before expiry to start warning
    warn_days: i64,

    #[structopt(name = "keys")]
    /// GPG key IDs in long format (40 uppercase hex characters, no spaces)
    keys: Vec<String>,
}

#[derive(Debug)]
struct GpgKeyStatus {
    subkey: bool,
    fingerprint: String,
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
            subkey: line1[0] == "sub",
            fingerprint: line2[9].to_string(),
            expires: line1[6].parse().ok(),
        }
    }
}

fn run() -> Result<i32, Error> {
    let opt = Opt::from_args();

    let args = {
        let mut v = vec!["--with-colons", "--fixed-list-mode", "--list-keys"];
        opt.keys.iter().for_each(|k| v.push(k));
        v
    };

    let output = String::from_utf8(
        Command::new("gpg")
            .args(args)
            .output()
            .expect("GPG command failed")
            .stdout,
    )?;
    let keys: Vec<GpgKeyStatus> = grouped(
        2,
        output
            .lines()
            .map(|line| line.split(":").collect::<Vec<&str>>())
            .filter(|line| line[0] == "pub" || line[0] == "sub" || line[0] == "fpr"),
    )
    .map(|group| (&group).into())
    .collect();

    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiring_keys: Vec<&GpgKeyStatus> = keys
        .iter()
        .filter(|key_status| opt.keys.contains(&key_status.fingerprint))
        .filter(|key_status| match key_status.expire_days(now_secs) {
            Some(days) => days <= (opt.warn_days as i64),
            None => false,
        })
        .collect();

    if expiring_keys.len() > 0 {
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

fn main() -> () {
    std::process::exit(run().unwrap());
}
