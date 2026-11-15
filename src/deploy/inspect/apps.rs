use super::{Release, inspect, valid_name};
use anyhow::{Result, bail};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Debug, Serialize)]
pub struct App {
    pub name: String,
    pub active: Option<Release>,
}
pub fn apps(root: &Path) -> Result<Vec<App>> {
    let path = root.join("metadata");
    if !path.exists() {
        return Ok(vec![]);
    }
    if !fs::symlink_metadata(&path)?.file_type().is_dir() {
        bail!("metadata directory is not a real directory");
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !valid_name(&name) {
            bail!("invalid managed app name");
        }
        if !entry.file_type()?.is_dir() {
            bail!("managed app directory is not regular");
        }
        result.push(App {
            active: inspect(root, &name)?,
            name,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}
