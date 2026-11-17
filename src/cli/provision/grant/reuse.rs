use super::{Arguments, SkryniaClient, secrets};
use anyhow::{Result, bail};
use serde_json::Value;
use std::path::Path;
pub fn issue(args: &Arguments, client: &SkryniaClient, path: &Path, existing: &str) -> Result<()> {
    for field in [
        "parent-grant-is-root",
        "parent-grant",
        "parent-token-file",
        "locations",
        "operations",
        "max-bytes",
        "max-objects",
    ] {
        if args.optional(field).is_some() {
            bail!("--existing-grant cannot be combined with --{field}");
        }
    }
    let record: Value = serde_json::from_str(&secrets::read(existing)?)?;
    let grant = record.get("grant").unwrap_or(&record);
    let id = grant["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("existing grant ID missing"))?
        .to_owned();
    if grant["issuerToken"].as_str().is_none() {
        bail!("existing grant issuer credential missing");
    }
    let saved = serde_json::json!({"state":"grant_saved","grant":grant});
    secrets::write_new(path, &serde_json::to_vec_pretty(&saved)?)?;
    super::super::issue::issue(client, path, grant)?;
    println!(
        "{}",
        serde_json::json!({"grantId":id,"credentialsFile":path,
        "hint":"new user token is stored privately at credentialsFile as .user.userToken"})
    );
    Ok(())
}
