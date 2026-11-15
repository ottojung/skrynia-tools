use super::{Release, files, valid_name};
use anyhow::{Result, bail};
use std::path::Path;

pub fn release(root: &Path, name: &str, id: &str) -> Result<Option<Release>> {
    if !valid_name(name) || !super::validate::valid_release_id(id) {
        bail!("invalid application name or release identifier");
    }
    if !files::existing_dir(&root.join("metadata").join(name))?
        || !files::existing_dir(&root.join("releases").join(name))?
    {
        return Ok(None);
    }
    let Some(entry) = super::read::metadata(root, name, id)? else {
        return Ok(None);
    };
    let tree = root.join("releases").join(name).join(id);
    if !files::existing_dir(&tree)? {
        return Ok(None);
    }
    Ok(Some(entry))
}
