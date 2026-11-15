use super::fixtures;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn verify() {
    let root = fixtures::root("plan");
    let token = root.join("parent-token");
    fixtures::private(&token, b"sensitive-parent-token");
    let journal = root.join("credentials.json");
    let file = journal.to_str().unwrap();
    let status = fixtures::command(&[
        "admin",
        "create",
        "--name",
        "test",
        "--url",
        "http://127.0.0.1:1",
        "--parent-token-file",
        token.to_str().unwrap(),
        "--credentials-out",
        file,
        "--max-bytes",
        "10000",
        "--max-objects",
        "50",
        "--id",
        "0123456789abcdef0123456789abcdef",
    ]);
    assert!(!status.status.success());
    let record: serde_json::Value = serde_json::from_slice(&fs::read(&journal).unwrap()).unwrap();
    assert_eq!(record["state"], "pending_grant_create");
    assert_eq!(record["prefix"], "a.test.0123456789abcdef0123456789abcdef.");
    assert_eq!(record["maxObjects"], 50);
    assert!(!String::from_utf8_lossy(&status.stderr).contains("sensitive-parent-token"));
    let inspect = fixtures::command(&["admin", "inspect", "--credentials-file", file]);
    assert!(inspect.status.success());
    assert!(
        String::from_utf8(inspect.stdout)
            .unwrap()
            .contains("pending_grant_create")
    );
    let refused = fixtures::command(&["admin", "resume", "--credentials-file", file]);
    assert!(!refused.status.success());
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&journal).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_dir_all(root).unwrap();
}
