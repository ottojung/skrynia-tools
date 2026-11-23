//! Disposable kernel filesystem layout, spawn and capability digest.
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub fn prepare(base: &Path, art: &Path, token: &[u8; 32]) -> io::Result<()> {
    if base.exists() {
        fs::remove_dir_all(base)?;
    }
    let op = base.join("operator");
    let us = base.join("user");
    fs::create_dir_all(op.join("security"))?;
    fs::create_dir_all(op.join("programs"))?;
    for s in ["active", "blob", "ready", "appdata"] {
        fs::create_dir_all(us.join("programs").join(s))?;
    }
    fs::write(op.join("security/program0.sha256"), sha256(token))?;
    fs::write(
        op.join("programs/root.wasm"),
        fs::read(art.join("p1_push_efe6117.wasm"))?,
    )?;
    fs::write(us.join("programs/operator.cap"), super::op_cap())?;
    Ok(())
}
pub fn spawn(art: &Path, base: &Path, port: u16, log: fs::File) -> io::Result<Child> {
    Command::new(art.join("kernel_PR125_856c029"))
        .env("SKRYNIA_ROOT_WASM", art.join("p0_d57ab27a.wasm"))
        .env("SKRYNIA_OPERATOR_DIR", base.join("operator"))
        .env("SKRYNIA_USER_DIR", base.join("user"))
        .env("SKRYNIA_BIND", format!("127.0.0.1:{port}"))
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()
}
pub fn sha256(b: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(b).into()
}
