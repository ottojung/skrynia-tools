use super::{Release, files, release, valid_name};
use anyhow::{Result, bail};
use std::{fs, path::Path};

pub fn releases(root: &Path, name: &str) -> Result<Vec<Release>> {
    if !valid_name(name) {
        bail!("invalid application name");
    }
    let directory = root.join("metadata").join(name);
    if !files::existing_dir(&directory)? {
        return Ok(vec![]);
    }
    let mut versions = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };
        let Some(id) = filename.strip_suffix(".json") else {
            continue;
        };
        if !super::validate::valid_release_id(id) {
            bail!("invalid release metadata filename");
        }
        if let Some(entry) = release(root, name, id)? {
            versions.push(entry);
        }
    }
    versions.sort_by(|a, b| {
        b.published_unix_ms
            .cmp(&a.published_unix_ms)
            .then_with(|| b.id.cmp(&a.id))
    });
    Ok(versions)
}
