//! Disposable kernel operator/user directory tree and root WASM staging.
use sha2::{Digest, Sha256};
use std::path::Path;
use std::{fs, io};

pub(super) fn prepare(base: &Path, art: &Path, token: &[u8; 32]) -> io::Result<()> {
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
        fs::read(art.join("p1_wake_18f9123a.wasm"))?,
    )?;
    fs::write(us.join("programs/operator.cap"), crate::op_cap())?;
    Ok(())
}
fn sha256(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
