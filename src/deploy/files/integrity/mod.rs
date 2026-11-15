use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
use std::path::Path;
mod file;
mod record;
#[cfg(test)]
mod tests;
mod walk;

pub fn digest_tree(root: &Path) -> Result<String> {
    if !std::fs::symlink_metadata(root)?.file_type().is_dir() {
        bail!("release root must be a real directory");
    }
    let mut hash = Sha256::new();
    hash.update(b"skrynia-release-tree-v1\0");
    walk::visit(root, Path::new(""), &mut hash)?;
    Ok(hex::encode(hash.finalize()))
}
