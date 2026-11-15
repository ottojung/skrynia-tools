use super::{files, process, types::Release, validate::valid_name};
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
mod active;
mod rollback;
pub use active::inspect;
pub use rollback::rollback;
