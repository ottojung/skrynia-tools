use super::group;
use anyhow::{Context, Result, bail};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn run(mut cmd: Command, seconds: u64) -> Result<()> {
    cmd.stdin(Stdio::null());
    // Duplicating fd 2 works for pipes as well as terminal/file stderr streams.
    use std::os::fd::FromRawFd;
    let fd = unsafe { libc::dup(libc::STDERR_FILENO) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    cmd.stdout(Stdio::from(unsafe { std::fs::File::from_raw_fd(fd) }));
    group::isolate(&mut cmd);
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
            group::terminate(&mut child);
            bail!("{description} timed out after {seconds} seconds");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
