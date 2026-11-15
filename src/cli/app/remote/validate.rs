use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

pub fn host(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 253
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        || !s.as_bytes()[0].is_ascii_alphanumeric()
        || !s.as_bytes()[s.len() - 1].is_ascii_alphanumeric()
    {
        bail!("invalid SSH host");
    }
    Ok(())
}
pub fn user(s: &str) -> Result<()> {
    let Some(first) = s.as_bytes().first() else {
        bail!("invalid SSH username");
    };
    if s.len() > 64
        || !(first.is_ascii_alphabetic() || *first == b'_')
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        bail!("invalid SSH username");
    }
    Ok(())
}
pub fn file(path: &str, private: bool) -> Result<()> {
    let info = fs::symlink_metadata(Path::new(path))?;
    if !info.file_type().is_file() || info.len() == 0 {
        bail!("invalid SSH key/host file");
    }
    #[cfg(unix)]
    if private && info.permissions().mode() & 0o077 != 0 {
        bail!("SSH private key is accessible to group or others");
    }
    Ok(())
}
