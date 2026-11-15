#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, process::Command};

#[test]
fn pending_grant_create_keeps_a_recoverable_plan() {
    let root = std::env::temp_dir().join(format!("skrynia-plan-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    let token = root.join("parent-token");
    fs::write(&token, "sensitive-parent-token").unwrap();
    #[cfg(unix)]
    fs::set_permissions(&token, fs::Permissions::from_mode(0o600)).unwrap();
    let journal = root.join("credentials.json");
    let status = Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args([
            "admin",
            "create",
            "--name",
            "test",
            "--url",
            "http://127.0.0.1:1",
            "--parent-token-file",
            token.to_str().unwrap(),
            "--credentials-out",
            journal.to_str().unwrap(),
            "--max-bytes",
            "10000",
            "--max-objects",
            "50",
            "--id",
            "0123456789abcdef0123456789abcdef",
        ])
        .output()
        .unwrap();
    assert!(!status.status.success());
    let record: serde_json::Value = serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();
    assert_eq!(record["state"], "pending_grant_create");
    assert_eq!(record["prefix"], "a.test.0123456789abcdef0123456789abcdef.");
    assert_eq!(record["maxObjects"], 50);
    assert!(!String::from_utf8_lossy(&status.stderr).contains("sensitive-parent-token"));
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&journal).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_dir_all(root).unwrap();
}
