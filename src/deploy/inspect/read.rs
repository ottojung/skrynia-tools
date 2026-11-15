use super::Release;
use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{
    fs::{self, OpenOptions},
    io::Read,
    path::Path,
};

pub fn metadata(root: &Path, name: &str, id: &str) -> Result<Option<Release>> {
    let path = root.join("metadata").join(name).join(format!("{id}.json"));
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = match options.open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let info = file.metadata()?;
    if !info.is_file() || fs::symlink_metadata(&path)?.file_type().is_symlink() {
        bail!("release metadata is not a regular file");
    }
    if info.len() > 16384 {
        bail!("release metadata exceeds 16 KiB");
    }
    let mut bytes = Vec::new();
    file.take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        bail!("release metadata exceeds 16 KiB");
    }
    let entry: Release = serde_json::from_slice(&bytes)?;
    if entry.name != name || entry.id != id || !entry.commit.starts_with(&id[..12]) {
        bail!("release metadata identity mismatch");
    }
    Ok(Some(entry))
}
