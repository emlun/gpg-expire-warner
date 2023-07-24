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

fn setup_homedir() -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("gpg-home");

    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;

    let gpg_cmd = || {
        let mut cmd = Command::new("gpg");
        cmd.arg("--homedir").arg(&dir);
        cmd
    };

    let gpg_create_key_cmd = || {
        let mut cmd = gpg_cmd();
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        cmd.args(["--batch"]).args(["--passphrase", ""]);
        cmd
    };

    assert!(
        gpg_create_key_cmd()
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
        gpg_cmd()
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
        gpg_create_key_cmd()
            .args(["--quick-add-key", main_id, "ed25519", "auth", "1d"])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create short expiry subkey"
    );

    assert!(
        gpg_create_key_cmd()
            .args(["--quick-add-key", main_id, "ed25519", "auth", "2d"])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create short expiry subkey that won't be checked"
    );

    assert!(
        gpg_create_key_cmd()
            .args(["--quick-add-key", main_id, "ed25519", "auth", "5d"])
            .spawn()?
            .wait()?
            .success(),
        "Failed to create long expiry subkey"
    );

    let list_keys_output = String::from_utf8(
        gpg_cmd()
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
        homedir: dir,
        short_id: key_ids[0].to_string(),
        short_unchecked_id: key_ids[1].to_string(),
        long_id: key_ids[2].to_string(),
    })
}

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnvironment {
        homedir,
        short_id,
        short_unchecked_id,
        long_id,
    } = setup_homedir()?;

    let prog_output = String::from_utf8(
        Command::new(env!("CARGO_BIN_EXE_gpg-expire-warner"))
            .args(["--days", "2", &short_id, &long_id])
            .env("GNUPGHOME", homedir)
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
