use super::{Arguments, Result, SkryniaClient, scan};
use anyhow::Context;
use std::{env, path::PathBuf};
pub fn token(args: &Arguments, client: &SkryniaClient) -> Result<String> {
    if args.optional("data-dir").is_some() {
        anyhow::bail!("use SKRYNIA_DATA_DIR, the same setting as skrynia-server");
    }
    let dir = env::var_os("SKRYNIA_DATA_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .context("--root-grant requires SKRYNIA_DATA_DIR, just like skrynia-server")?;
    let (id, issuer) = scan::find(&dir)?;
    let user = client.issue_user(&id, &issuer)?;
    user["userToken"]
        .as_str()
        .map(str::to_owned)
        .context("root user token missing from server response")
}
