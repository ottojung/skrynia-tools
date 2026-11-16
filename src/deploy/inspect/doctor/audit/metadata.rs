use super::{Audit, files, release, validate};
use anyhow::Result;
use std::{fs, path::Path};
pub fn scan(root: &Path, name: &str, out: &mut Audit) -> Result<()> {
    let directory = root.join("metadata").join(name);
    if !files::existing_dir(&directory)? {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = filename.strip_suffix(".json") else {
            out.invalid_entries.push(format!("metadata/{filename}"));
            continue;
        };
        if !validate::valid_release_id(id) || !entry.file_type()?.is_file() {
            out.invalid_entries.push(format!("metadata/{filename}"));
            continue;
        }
        match release(root, name, id) {
            Ok(Some(_)) => out.available_releases += 1,
            Ok(None) => out.metadata_without_artifact.push(id.into()),
            Err(_) => out.invalid_entries.push(format!("metadata/{filename}")),
        }
    }
    Ok(())
}
