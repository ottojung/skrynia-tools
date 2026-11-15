use super::Fixture;
use std::{fs, process::Command};
pub fn commit(f: &Fixture, text: &str) -> String {
    fs::write(f.source.join("shared.txt"), text).unwrap();
    let status = Command::new("git")
        .current_dir(&f.source)
        .args(["add", "."])
        .status()
        .unwrap();
    assert!(status.success());
    let output = Command::new("git")
        .current_dir(&f.source)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "fixture",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let output = Command::new("git")
        .current_dir(&f.source)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
