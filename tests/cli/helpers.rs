use std::process::{Command, Output};
pub fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args(args)
        .env_remove("SKRYNIA_URL")
        .env_remove("SKRYNIA_PARENT_GRANT")
        .output()
        .unwrap()
}
