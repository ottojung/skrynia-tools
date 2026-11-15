use anyhow::Result;
#[cfg(unix)]
pub fn lock(root: &std::path::Path, name: &str) -> Result<std::fs::File> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(root.join(".locks").join(name))?;
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(file)
}
