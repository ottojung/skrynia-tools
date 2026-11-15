mod support;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use support::Fixture;

#[test]
fn restricted_remote_deploy_cannot_publish_elsewhere() {
    let fixture = Fixture::new();
    let commit = fixture.commit("root");
    std::fs::create_dir_all(&fixture.apps).unwrap();
    let other = fixture.root.join("other");
    std::fs::create_dir_all(&other).unwrap();
    let spec = serde_json::json!({"name":"hello", "repo":"https://github.com/owner/repo",
        "subdir":"apps/site","commit":commit,"apps_dir":other,"builder_image":"fake-builder"});
    let mut child = Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args(["app", "deploy", "--spec-stdin", "true"])
        .env("SKRYNIA_DEPLOY_ROOT", &fixture.apps)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(spec.to_string().as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("deployment root differs"));
    assert!(!other.join("apps").exists());
}
