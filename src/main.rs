use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use toml;

#[derive(Debug)]
enum Error {
    IoError(std::io::Error),
    TimeError(std::time::SystemTimeError),
    TomlError(toml::de::Error),
    Utf8Error(std::string::FromUtf8Error),
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}
impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::TimeError(e)
    }
}
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Error {
        Error::TomlError(e)
    }
}
impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Error {
        Error::Utf8Error(e)
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    keys: BTreeSet<String>,
    warn_days: u64,
}

fn read_config() -> Result<Config, Error> {
    let mut contents: String = String::new();
    let mut file = File::open(
        ProjectDirs::from("", "", "gpg-expire-warner")
            .expect("Failed to locate config directory")
            .config_dir()
            .join("config.toml"),
    )?;
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
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

struct Grouped<I, T>
where
    I: Iterator<Item = T>,
{
    n: usize,
    it: I,
}
impl<I, T> Iterator for Grouped<I, T>
where
    I: Iterator<Item = T>,
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Vec<T>> {
        let mut nexts: Vec<T> = Vec::new();

        loop {
            match self.it.next() {
                Some(next) => nexts.push(next),
                None => break,
            };
            if nexts.len() == self.n {
                break;
            }
        }
        if nexts.is_empty() {
            None
        } else {
            Some(nexts)
        }
    }
}

fn grouped<T, I: Iterator<Item = T>>(n: usize, it: I) -> Grouped<I, T> {
    Grouped { n: n, it: it }
}

fn run() -> Result<i32, Error> {
    let config = read_config()?;

    let args = {
        let mut v = vec!["--with-colons", "--fixed-list-mode", "--list-keys"];
        config.keys.iter().for_each(|k| v.push(k));
        v
    };

    let output = Command::new("gpg")
        .args(args)
        .output()
        .expect("GPG command failed");
    let output = String::from_utf8(output.stdout)?;
    let keys: Vec<GpgKeyStatus> = grouped(
        2,
        output
            .lines()
            .map(|line| line.split(":").collect::<Vec<&str>>())
            .filter(|line| line[0] == "pub" || line[0] == "sub" || line[0] == "fpr"),
    )
    .map(|group| (&group).into())
    .collect();

    let matched_keys: Vec<&GpgKeyStatus> = keys
        .iter()
        .filter(|key_status| config.keys.contains(&key_status.fingerprint))
        .collect();

    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let expiring_keys: Vec<&&GpgKeyStatus> = matched_keys
        .iter()
        .filter(|key_status| match key_status.expire_days(now_secs) {
            Some(days) => days <= (config.warn_days as i64),
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
    }

    Ok(0)
}

fn main() -> () {
    std::process::exit(run().unwrap());
}
