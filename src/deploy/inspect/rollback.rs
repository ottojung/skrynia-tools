use super::{files, releases, valid_name};
use anyhow::{Result, bail};
use std::path::Path;
pub fn rollback(root: &Path, name: &str, id: &str) -> Result<()> {
    if !valid_name(name) {
        bail!("invalid app name");
    }
    files::ensure_dirs(root)?;
    let _lock = super::process::lock(root, name)?;
    if !releases(root, name)?.iter().any(|r| r.id == id)
        || !root.join("releases").join(name).join(id).is_dir()
    {
        bail!("requested release is not available");
    }
    files::switch(root, name, id)
}
