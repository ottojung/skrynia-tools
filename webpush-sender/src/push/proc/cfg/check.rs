use super::{Error, Result, self_uid};
pub(super) const MAX_CONFIG_BYTES: u64 = 64 * 1024;

#[cfg(unix)]
pub(super) fn check(meta: &std::fs::Metadata) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if meta.len() > MAX_CONFIG_BYTES {
        return Err(Error::Config("config-size"));
    }
    if !meta.is_file() {
        return Err(Error::Config("config-kind"));
    }
    if meta.uid() != self_uid()? {
        return Err(Error::Config("config-owner"));
    }
    if meta.nlink() != 1 {
        return Err(Error::Config("config-links"));
    }
    if meta.permissions().mode() & 0o077 != 0 {
        return Err(Error::Config("config-mode"));
    }
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn check(meta: &std::fs::Metadata) -> Result<()> {
    if meta.len() > MAX_CONFIG_BYTES {
        return Err(Error::Config("config-size"));
    }
    if !meta.is_file() {
        return Err(Error::Config("config-kind"));
    }
    Ok(())
}
