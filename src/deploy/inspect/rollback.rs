use super::{files, release, valid_name};
use anyhow::{Result, bail};
use std::path::Path;
pub fn rollback(root: &Path, name: &str, id: &str) -> Result<()> {
    if !valid_name(name) {
        bail!("invalid app name");
    }
    files::ensure_dirs(root)?;
    let _lock = super::process::lock(root, name)?;
    if release(root, name, id)?.is_none() {
        bail!("requested release is not available");
    }
    files::switch(root, name, id)
}
