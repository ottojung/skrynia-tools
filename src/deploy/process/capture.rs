use anyhow::{Context, Result, bail};
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

const MAX_OUTPUT: u64 = 65536;

pub fn capture(mut cmd: Command, seconds: u64) -> Result<Vec<u8>> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped());
    let mut child = cmd.spawn()?;
    let reader = child.stdout.take().context("stdout missing")?;
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = reader
            .take(MAX_OUTPUT + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = tx.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        if let Some(status) = child.try_wait()? {
            let left = deadline.saturating_duration_since(Instant::now());
            let bytes = rx
                .recv_timeout(left)
                .context("subprocess stdout remained open")??;
            if bytes.len() as u64 > MAX_OUTPUT {
                bail!("subprocess output exceeds 64 KiB");
            }
            if !status.success() {
                bail!("subprocess failed with {status}");
            }
            return Ok(bytes);
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            bail!("subprocess timed out after {seconds} seconds");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
