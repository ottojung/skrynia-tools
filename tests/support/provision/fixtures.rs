#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub fn root(label: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("skrynia-{label}-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    root
}
pub fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
}
pub fn command(parts: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_skrynia-tools"))
        .args(parts)
        .output()
        .unwrap()
}
