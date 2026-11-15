use anyhow::{Context, Result, bail};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn run(mut cmd: Command, seconds: u64) -> Result<()> {
    cmd.stdin(Stdio::null());
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
