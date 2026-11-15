use super::{Arguments, options};
use anyhow::{Result, bail};
use std::{
    io::Write,
    process::Stdio,
    time::{Duration, Instant},
};

pub fn run(args: &Arguments, payload: &[u8]) -> Result<()> {
    if payload.len() > 16384 {
        bail!("remote request exceeds 16 KiB");
    }
    let mut command = options::command(args)?;
    let mut child = command.stdin(Stdio::piped()).spawn()?;
    let result = child.stdin.take().unwrap().write_all(payload);
    if let Err(error) = result {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.into());
    }
    let limit = if args.action == "deploy" { 1800 } else { 120 };
    let deadline = Instant::now() + Duration::from_secs(limit);
    loop {
        if let Some(status) = child.try_wait()? {
            if status.success() {
                return Ok(());
            }
            bail!("remote app operation failed ({status}); check active state before retrying");
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            bail!("SSH operation timed out; remote changes may have committed");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
