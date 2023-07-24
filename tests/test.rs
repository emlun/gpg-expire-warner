use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

struct TestEnvironment {
    homedir: PathBuf,
    short_id: String,
    short_unchecked_id: String,
    long_id: String,
}

impl TestEnvironment {
    fn gpg_cmd(&self) -> Command {
        let mut cmd = Command::new("gpg");
        cmd.arg("--homedir").arg(&self.homedir);
        cmd
    }

    fn gpg_create_key_cmd(&self) -> Command {
        let mut cmd = self.gpg_cmd();
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        cmd.args(["--batch"]).args(["--passphrase", ""]);
        cmd
    }
}

fn setup_homedir<P>(test_name: P) -> Result<TestEnvironment, Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(test_name)
        .join("gpg-home");

    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;

    let test_env = TestEnvironment {
        homedir: dir,
        short_id: "".to_string(),
        short_unchecked_id: "".to_string(),
        long_id: "".to_string(),
    };

    assert!(
        test_env
            .gpg_create_key_cmd()
            .args([
                "--quick-generate-key",
                "test@example.org",
                "ed25519",
                "sign",
                "1d",
            ])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create main key"
    );

    let list_keys_output = String::from_utf8(
        test_env
            .gpg_cmd()
            .args(["--with-colons", "--fixed-list-mode", "--list-keys"])
            .output()
            .expect("GPG command failed")
            .stdout,
    )?;

    let main_id = list_keys_output
        .lines()
        .map(|line| line.split(':').collect::<Vec<&str>>())
        .skip_while(|line| line[0] != "pub")
        .find(|line| line[0] == "fpr")
        .map(|line| line[9])
        .unwrap();

    assert!(
        test_env
            .gpg_create_key_cmd()
            .args(["--quick-add-key", main_id, "ed25519", "auth", "1d"])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create short expiry subkey"
    );

    assert!(
        test_env
            .gpg_create_key_cmd()
            .args(["--quick-add-key", main_id, "ed25519", "auth", "2d"])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create short expiry subkey that won't be checked"
    );

    assert!(
        test_env
            .gpg_create_key_cmd()
            .args(["--quick-add-key", main_id, "ed25519", "auth", "5d"])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create long expiry subkey"
    );

    let list_keys_output = String::from_utf8(
        test_env
            .gpg_cmd()
            .args(["--with-colons", "--fixed-list-mode", "--list-keys"])
            .output()
            .expect("GPG command failed")
            .stdout,
    )?;

    let mut subkeys: Vec<(&str, u64)> = list_keys_output
        .lines()
        .map(|line| line.split(':').collect::<Vec<&str>>())
        .filter(|line| line[0] == "sub")
        .map(|line| {
            (
                line[4],
                line[6].parse::<u64>().expect("Failed to parse expiry time"),
            )
        })
        .collect();
    subkeys.sort_by_key(|(_, expire)| *expire);

    let key_ids: Vec<&str> = subkeys
        .into_iter()
        .map(|(id, _)| id)
        .flat_map(|id| {
            list_keys_output
                .lines()
                .map(|line| line.split(':').collect::<Vec<&str>>())
                .find(|line| line[0] == "fpr" && line[9].ends_with(id))
                .map(|line| line[9])
        })
        .collect();

    Ok(TestEnvironment {
        short_id: key_ids[0].to_string(),
        short_unchecked_id: key_ids[1].to_string(),
        long_id: key_ids[2].to_string(),
        ..test_env
    })
}

#[test]
fn test_default() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnvironment {
        homedir,
        short_id,
        short_unchecked_id,
        long_id,
    } = setup_homedir("test_default")?;

    let prog_output = String::from_utf8(
        Command::new(env!("CARGO_BIN_EXE_gpg-expire-warner"))
            .env("GNUPGHOME", homedir)
            .args(["--days", "2", &short_id, &long_id])
            .output()?
            .stdout,
    )?;

    assert!(
        prog_output.contains(&format!("{short_id}: 1 days"))
            || prog_output.contains(&format!("{short_id}: 0 days")),
        "Short expiry key {} not found in output: {}",
        short_id,
        prog_output,
    );
    assert!(
        !prog_output.contains(&short_unchecked_id),
        "Unchecked short expiry key {} found in output: {}",
        short_unchecked_id,
        prog_output,
    );
    assert!(
        !prog_output.contains(&long_id),
        "Long expiry key {} found in output: {}",
        long_id,
        prog_output,
    );
    Ok(())
}

#[test]
fn test_expand() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnvironment {
        homedir,
        short_id,
        short_unchecked_id,
        long_id,
    } = setup_homedir("test_expand")?;

    assert!(
        Command::new(env!("CARGO_BIN_EXE_gpg-expire-warner"))
            .env("GNUPGHOME", &homedir)
            .args(["--days", "2", &short_id, &long_id, "--extend", "10d"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?
            .wait()?
            .success(),
        "Failed to extend validity",
    );

    let prog_output = String::from_utf8(
        Command::new(env!("CARGO_BIN_EXE_gpg-expire-warner"))
            .env("GNUPGHOME", &homedir)
            .args(["--days", "7", &short_id, &long_id])
            .output()?
            .stdout,
    )?;

    assert!(
        !prog_output.contains(&short_id),
        "Short expiry key {} found in output: {}",
        short_id,
        prog_output,
    );
    assert!(
        !prog_output.contains(&short_unchecked_id),
        "Unchecked short expiry key {} found in output: {}",
        short_unchecked_id,
        prog_output,
    );
    assert!(
        prog_output.contains(&long_id),
        "Long expiry key {} not found in output: {}",
        long_id,
        prog_output,
    );
    Ok(())
}
