use anyhow::{Result, bail};
#[cfg(unix)]
use std::{
    fs::File,
    os::{
        fd::AsRawFd,
        unix::fs::{OpenOptionsExt, PermissionsExt},
    },
};
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

#[cfg(unix)]
pub fn lock(path: &Path) -> Result<File> {
    let mut name = path.as_os_str().to_os_string();
    name.push(".lock");
    let lock = PathBuf::from(name);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(lock)?;
    let info = file.metadata()?;
    if !info.file_type().is_file() || info.permissions().mode() & 0o077 != 0 {
        bail!("credential lock is not a private regular file");
    }
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(file)
}
