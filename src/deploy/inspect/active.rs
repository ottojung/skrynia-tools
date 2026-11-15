use super::{Release, releases, valid_name};
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn inspect(root: &Path, name: &str) -> Result<Option<Release>> {
    if !valid_name(name) {
        bail!("invalid app name");
    }
    let path = root.join("apps").join(name);
    let kind = match fs::symlink_metadata(&path) {
        Ok(meta) => meta.file_type(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !kind.is_symlink() {
        bail!("active application is not a release symlink");
    }
    let target = fs::read_link(&path)?;
    let id = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid release link"))?;
    let expected = Path::new("..").join("releases").join(name).join(id);
    if target != expected {
        bail!("release link escapes this application");
    }
    releases(root, name)?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| anyhow::anyhow!("active release has no verified metadata"))
        .map(Some)
}
