use super::{Release, files};
use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{fs::OpenOptions, io::Write, path::Path};

pub fn save(directory: &Path, entry: &Release) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(entry)?;
    if bytes.len() > 16384 {
        bail!("release record exceeds 16 KiB limit");
    }
    let path = directory.join(format!("{}.json", entry.id));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    let mut file = options.open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    files::sync_dir(directory)
}
