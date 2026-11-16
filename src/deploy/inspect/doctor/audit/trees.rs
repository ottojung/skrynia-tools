use super::{Audit, files, validate};
use anyhow::Result;
use std::{fs, path::Path};
pub fn scan(root: &Path, name: &str, out: &mut Audit) -> Result<()> {
    let directory = root.join("releases").join(name);
    if !files::existing_dir(&directory)? {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(id) = name
            .strip_prefix('.')
            .and_then(|s| s.strip_suffix(".staging"))
        {
            if validate::valid_release_id(id) && entry.file_type()?.is_dir() {
                out.abandoned_staging.push(id.into());
            } else {
                out.invalid_entries.push(format!("releases/{name}"));
            }
        } else if validate::valid_release_id(&name) && entry.file_type()?.is_dir() {
            let target = root
                .join("metadata")
                .join(out.name.as_str())
                .join(format!("{name}.json"));
            if !target.exists() {
                out.artifact_without_metadata.push(name);
            }
        } else {
            out.invalid_entries.push(format!("releases/{name}"));
        }
    }
    Ok(())
}
