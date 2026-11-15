use anyhow::{Result, bail};
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub fn capture(mut cmd: Command, seconds: u64) -> Result<Vec<u8>> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped());
    let mut child = cmd.spawn()?;
    let limit = Instant::now() + Duration::from_secs(seconds);
    loop {
        if let Some(status) = child.try_wait()? {
            if !status.success() {
                bail!("subprocess failed with {status}");
            }
            let mut result = Vec::new();
            child.stdout.take().unwrap().read_to_end(&mut result)?;
            return Ok(result);
        }
        if Instant::now() >= limit {
            child.kill()?;
            let _ = child.wait();
            bail!("subprocess timed out after {seconds} seconds");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
