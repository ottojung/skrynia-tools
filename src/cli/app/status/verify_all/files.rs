use super::{Result, bail, deploy};
use std::{fs, path::Path};

pub fn names(root: &Path, name: &str) -> Result<Vec<String>> {
    if !deploy::valid_name(name) {
        bail!("invalid application name");
    }
    let directory = root.join("metadata").join(name);
    let kind = fs::symlink_metadata(&directory)?;
    if !kind.file_type().is_dir() {
        bail!("release metadata root must be a real directory");
    }
    let mut names = fs::read_dir(directory)?
        .map(|entry| entry.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()?;
    if names.is_empty() {
        bail!("no retained releases to verify");
    }
    names.sort();
    Ok(names)
}
