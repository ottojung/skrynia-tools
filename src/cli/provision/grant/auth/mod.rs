mod root;
mod scan;
use super::{Arguments, SkryniaClient, secrets};
use anyhow::{Result, bail};
use serde_json::Value;
pub fn parent(args: &Arguments, client: &SkryniaClient) -> Result<String> {
    if args.optional("root-grant").is_some() {
        if args.optional("parent-grant").is_some() || args.optional("parent-token-file").is_some() {
            bail!("--root-grant conflicts with --parent-grant/--parent-token-file");
        }
        return root::token(args, client);
    }
    let source = args
        .optional("parent-grant")
        .map(str::to_owned)
        .or_else(|| std::env::var("SKRYNIA_PARENT_GRANT").ok());
    if let Some(file) = source {
        let record: Value = serde_json::from_str(&secrets::read(&file)?)?;
        return record["user"]["userToken"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| anyhow::anyhow!("parent grant lacks saved user token"));
    }
    if let Some(file) = args.optional("parent-token-file") {
        return secrets::read(file);
    }
    bail!("supply --parent-grant FILE (or SKRYNIA_PARENT_GRANT), or --root-grant")
}
