use super::{Audit, files};
use anyhow::Result;
use std::{fs, path::Path};
pub fn scan(root: &Path, name: &str, out: &mut Audit) -> Result<()> {
    let directory = root.join("apps");
    if !files::existing_dir(&directory)? {
        return Ok(());
    }
    let prefix = format!(".{name}.");
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        let Some(id) = filename.strip_prefix(&prefix) else {
            continue;
        };
        if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        if entry.file_type()?.is_symlink() {
            out.abandoned_activation_links.push(filename);
        } else {
            out.invalid_entries.push(format!("apps/{filename}"));
        }
    }
    Ok(())
}
