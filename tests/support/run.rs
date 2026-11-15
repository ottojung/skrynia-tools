use super::Fixture;
use std::process::{Command, Output};
// These command helpers are intentionally shared by different integration binaries.
#[allow(dead_code)]
impl Fixture {
    pub fn deploy(&self, commit: &str) -> Output {
        self.cli(&[
            "app",
            "deploy",
            "--name",
            "hello",
            "--repo",
            "https://github.com/owner/repo",
            "--subdir",
            "apps/site",
            "--commit",
            commit,
            "--builder-image",
            "fake-builder",
            "--apps-dir",
            self.apps.to_str().unwrap(),
        ])
    }
    pub fn cli(&self, args: &[&str]) -> Output {
        let path = std::env::var("PATH").unwrap();
        Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
            .args(args)
            .env("GIT_CONFIG_GLOBAL", self.root.join("gitconfig"))
            .env(
                "PATH",
                format!("{}:{path}", self.root.join("bin").display()),
            )
            .output()
            .unwrap()
    }
}
