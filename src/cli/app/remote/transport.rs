use crate::group;
use anyhow::{Result, bail};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

pub fn send(mut command: Command, payload: &[u8], seconds: u64) -> Result<()> {
    command.stdin(Stdio::piped());
    group::isolate(&mut command);
    let mut child = command.spawn()?;
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("SSH stdin missing"))?;
    let data = payload.to_vec();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = sender.send(input.write_all(&data));
    });
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        if let Some(status) = child.try_wait()? {
            let left = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(left) {
                Ok(Ok(())) if status.success() => return Ok(()),
                Ok(Ok(())) => bail!(
                    "remote app operation failed ({status}); check active state before retrying"
                ),
                Ok(Err(error)) => {
                    bail!("SSH stdin failed ({error}); remote changes may have committed")
                }
                Err(_) => {
                    group::terminate(&mut child);
                    bail!("SSH input delivery unconfirmed; remote changes may have committed");
                }
            }
        }
        if Instant::now() >= deadline {
            group::terminate(&mut child);
            bail!("SSH operation timed out; remote changes may have committed");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
