mod support;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use support::Fixture;

#[test]
fn deploy_from_stdin_is_same_as_cli_arguments() {
    let fixture = Fixture::new();
    let commit = fixture.commit("stdin success");
    let spec = serde_json::json!({"name":"hello", "repo":"https://github.com/owner/repo",
        "subdir":"apps/site","commit":commit,
        "apps_dir":fixture.apps, "builder_image":"fake-builder"});
    let path = std::env::var("PATH").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args(["app", "deploy", "--spec-stdin", "true"])
        .env("GIT_CONFIG_GLOBAL", fixture.root.join("gitconfig"))
        .env(
            "PATH",
            format!("{}:{path}", fixture.root.join("bin").display()),
        )
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
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let published: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(published["commit"], spec["commit"]);
    assert_eq!(
        std::fs::read_to_string(fixture.apps.join("apps/hello/index.html")).unwrap(),
        "stdin success"
    );
}
