use super::{files, types::Release, validate::valid_name};
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn releases(root: &Path, name: &str) -> Result<Vec<Release>> {
    if !valid_name(name) {
        bail!("invalid app name");
    }
    let folder = root.join("metadata").join(name);
    if !folder.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for file in fs::read_dir(folder)? {
        let file = file?;
        if file.path().extension().is_some_and(|ext| ext == "json") {
            let entry: Release = serde_json::from_slice(&fs::read(file.path())?)?;
            if root.join("releases").join(name).join(&entry.id).is_dir() {
                out.push(entry);
            }
        }
    }
    out.sort_by(|a: &Release, b: &Release| a.id.cmp(&b.id));
    Ok(out)
}
pub fn inspect(root: &Path, name: &str) -> Result<Option<Release>> {
    if !valid_name(name) {
        bail!("invalid app name");
    }
    let path = root.join("apps").join(name);
    if fs::symlink_metadata(&path).is_err() {
        return Ok(None);
    }
    let target = fs::read_link(&path)?;
    let Some(id) = target.file_name().and_then(|x| x.to_str()) else {
        bail!("invalid active release link");
    };
    Ok(releases(root, name)?.into_iter().find(|r| r.id == id))
}
pub fn rollback(root: &Path, name: &str, id: &str) -> Result<()> {
    if !valid_name(name)
        || !releases(root, name)?.iter().any(|r| r.id == id)
        || !root.join("releases").join(name).join(id).is_dir()
    {
        bail!("requested release is not available");
    }
    super::validate::ensure_dirs(root)?;
    let _lock = super::process::lock(root, name)?;
    files::switch(root, name, id)
}
