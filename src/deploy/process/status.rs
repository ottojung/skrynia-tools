use super::group;
use anyhow::Result;
use std::{
    process::{Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

/// Return an exit code (including ordinary nonzero status) with a hard deadline.
pub fn status(mut cmd: Command, seconds: u64) -> Result<ExitStatus> {
    cmd.stdin(Stdio::null()).stdout(Stdio::null());
    group::isolate(&mut cmd);
    let mut child = cmd.spawn()?;
    let limit = Instant::now() + Duration::from_secs(seconds);
    loop {
        if let Some(exit) = child.try_wait()? {
            return Ok(exit);
        }
        if Instant::now() >= limit {
            group::terminate(&mut child);
            anyhow::bail!("subprocess status probe timed out after {seconds} seconds");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
