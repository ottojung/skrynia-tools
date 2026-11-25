use super::{Error, Result, self_uid};
use std::fs::File;

#[cfg(unix)]
pub(super) fn check(file: &File) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let m = file.metadata().map_err(|_| Error::Config("lock-stat"))?;
    if !m.is_file() {
        return Err(Error::Config("lock-kind"));
    }
    if m.uid() != self_uid()? {
        return Err(Error::Config("lock-owner"));
    }
    if m.nlink() != 1 {
        return Err(Error::Config("lock-links"));
    }
    if m.permissions().mode() & 0o077 != 0 {
        return Err(Error::Config("lock-mode"));
    }
    if m.len() != 0 {
        return Err(Error::Config("lock-nonempty"));
    }
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn check(file: &File) -> Result<()> {
    let m = file.metadata().map_err(|_| Error::Config("lock-stat"))?;
    if !m.is_file() {
        return Err(Error::Config("lock-kind"));
    }
    Ok(())
}
