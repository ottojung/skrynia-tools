use super::{Arguments, policy, validate};
use anyhow::{Result, bail};
use std::process::Command;

pub fn command(args: &Arguments) -> Result<Command> {
    let host = args.required("ssh-host")?;
    let user = args.required("ssh-user")?;
    let key = args.required("ssh-key")?;
    let known = args.required("ssh-known-hosts")?;
    validate::host(host)?;
    validate::user(user)?;
    validate::file(key, true)?;
    validate::file(known, false)?;
    let port: u16 = args.optional("ssh-port").unwrap_or("22").parse()?;
    if port == 0 {
        bail!("invalid SSH port");
    }
    let mut cmd = Command::new("ssh");
    cmd.args(["-F", "/dev/null", "-T", "-p", &port.to_string(), "-i", key]);
    policy::apply(&mut cmd);
    cmd.arg("-o").arg(format!("UserKnownHostsFile={known}"));
    cmd.arg("--").arg(format!("{user}@{host}"));
    cmd.arg("skrynia-tools app rpc --spec-stdin true");
    Ok(cmd)
}
