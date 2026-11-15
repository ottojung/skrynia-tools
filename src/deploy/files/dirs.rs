use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn checked_dir(path: &Path) -> Result<()> {
    if fs::symlink_metadata(path).is_err() {
        fs::create_dir_all(path)?;
    }
    let kind = fs::symlink_metadata(path)?.file_type();
    if !kind.is_dir() || kind.is_symlink() {
        bail!(
            "deployment directory must be a real directory: {}",
            path.display()
        );
    }
    Ok(())
}
pub fn ensure_dirs(root: &Path) -> Result<()> {
    checked_dir(root)?;
    for sub in ["apps", "releases", "metadata", ".work", ".locks"] {
        checked_dir(&root.join(sub))?;
    }
    Ok(())
}
