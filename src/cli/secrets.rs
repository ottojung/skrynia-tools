use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn read(path: &str) -> Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        bail!("secret source is not a regular file");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!("secret file must not be accessible to group or others");
    }
    let value = fs::read_to_string(path)?;
    if value.trim().is_empty() {
        bail!("empty token file");
    }
    Ok(value.trim().into())
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut file = opts.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub fn replace(path: &Path, bytes: &[u8]) -> Result<()> {
    let candidate = path.with_extension(format!("secret-{:016x}", rand::random::<u64>()));
    write_new(&candidate, bytes)?;
    if let Err(error) = fs::rename(&candidate, path) {
        let _ = fs::remove_file(candidate);
        return Err(error.into());
    }
    Ok(())
}
