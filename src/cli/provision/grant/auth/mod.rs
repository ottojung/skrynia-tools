mod root;
mod scan;
use super::{Arguments, SkryniaClient};
use anyhow::{Result, bail};
pub fn parent(args: &Arguments, client: &SkryniaClient) -> Result<String> {
    if args.optional("parent-token-file").is_some() {
        bail!("admin grant accepts a token via --parent-grant, not a file");
    }
    if args.optional("parent-grant-is-root").is_some() {
        if args.optional("parent-grant").is_some() {
            bail!("--parent-grant-is-root conflicts with --parent-grant");
        }
        return root::token(args, client);
    }
    let token = args
        .optional("parent-grant")
        .map(str::to_owned)
        .or_else(|| std::env::var("SKRYNIA_PARENT_GRANT").ok());
    let token = token.ok_or_else(|| {
        anyhow::anyhow!(
            "supply --parent-grant TOKEN (or SKRYNIA_PARENT_GRANT), or --parent-grant-is-root"
        )
    })?;
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("--parent-grant must be a Skrynia grant user token, not a path");
    }
    Ok(token)
}
