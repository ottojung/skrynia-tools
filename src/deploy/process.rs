use anyhow::{Context, Result, bail};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn run(mut cmd: Command, seconds: u64) -> Result<()> {
    cmd.stdin(Stdio::null());
    // Preserve machine-readable CLI stdout by sending build logs to stderr.
    cmd.stdout(Stdio::from(
        std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/stderr")?,
    ));
    let description = format!("{:?}", cmd.get_program());
    let mut child = cmd
        .spawn()
        .with_context(|| format!("starting {description}"))?;
    let until = Instant::now() + Duration::from_secs(seconds);
    loop {
        if let Some(status) = child.try_wait()? {
            if status.success() {
                return Ok(());
            }
            bail!("{description} failed with {status}");
        }
        if Instant::now() >= until {
            child.kill()?;
            let _ = child.wait();
            bail!("{description} timed out after {seconds} seconds");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

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
