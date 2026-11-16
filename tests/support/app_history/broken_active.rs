use super::Fixture;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(unix)]
pub fn verify(fixture: &Fixture) {
    let root = fixture.apps.to_str().unwrap();
    let link = fixture.apps.join("apps/hello");
    let original = fs::read_link(&link).unwrap();
    fs::remove_file(&link).unwrap();
    symlink("../releases/hello/000000000000-0000000000000000", &link).unwrap();
    let output = fixture.cli(&[
        "app",
        "audit",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--format",
        "json",
    ]);
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["clean"], false);
    assert!(
        report["active_error"]
            .as_str()
            .unwrap()
            .contains("active release")
    );
    assert_eq!(report["active_release"], serde_json::Value::Null);
    fs::remove_file(&link).unwrap();
    symlink(original, link).unwrap();
}
