use super::parent;
use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    parent::sync(path)
}
pub fn replace(path: &Path, bytes: &[u8]) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        bail!("secret target is not a regular file");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!("secret target is not private");
    }
    let candidate = path.with_extension(format!("secret-{:032x}", rand::random::<u128>()));
    write_new(&candidate, bytes)?;
    if let Err(error) = fs::rename(&candidate, path) {
        let _ = fs::remove_file(&candidate);
        return Err(error.into());
    }
    parent::sync(path)
}
