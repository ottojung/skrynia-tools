use super::{Arguments, options, transport};
use anyhow::{Result, bail};

pub fn run(args: &Arguments, payload: &[u8]) -> Result<()> {
    if payload.len() > 16384 {
        bail!("remote request exceeds 16 KiB");
    }
    let command = options::command(args)?;
    let deadline_seconds = if args.action == "deploy" { 1800 } else { 120 };
    transport::send(command, payload, deadline_seconds)
}
