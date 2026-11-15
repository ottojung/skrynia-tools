use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{fs::OpenOptions, io::Read};

pub fn read(path: &str) -> Result<String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.file_type().is_file() {
        bail!("secret source must be a regular file");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        bail!("secret source is accessible to group or others");
    }
    if metadata.len() > 8192 {
        bail!("secret source is too large");
    }
    let mut contents = String::new();
    file.take(8193).read_to_string(&mut contents)?;
    if contents.len() > 8192 {
        bail!("secret source is too large");
    }
    if contents.trim().is_empty() {
        bail!("empty secret source");
    }
    Ok(contents.trim().into())
}
