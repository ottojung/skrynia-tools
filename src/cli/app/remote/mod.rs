use super::{Arguments, publish, rpc};
use anyhow::{Result, bail};
mod options;
mod policy;
mod spec;
mod ssh;
mod validate;

pub fn run(args: &Arguments) -> Result<()> {
    let payload = spec::encode(args)?;
    ssh::run(args, &payload)
}

pub fn requested(args: &Arguments) -> Result<bool> {
    let has_host = args.optional("ssh-host").is_some();
    if !has_host
        && ["ssh-user", "ssh-key", "ssh-known-hosts", "ssh-port"]
            .iter()
            .any(|name| args.optional(name).is_some())
    {
        bail!("--ssh-host is required when using SSH options");
    }
    Ok(has_host)
}
