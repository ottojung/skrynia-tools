use super::{Result, SkryniaClient, Value, secrets};
use std::path::Path;

pub fn issue(client: &SkryniaClient, path: &Path, grant: &Value) -> Result<()> {
    let id = grant["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("grant ID missing"))?;
    let token = grant["issuerToken"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("issuer token missing"))?;
    let marker = serde_json::json!({"state":"issuing_user","grant":grant});
    secrets::replace(path, serde_json::to_vec_pretty(&marker)?.as_slice())?;
    let user = client.issue_user(id, token)?;
    let complete = serde_json::json!({"grant":grant,"user":user});
    secrets::replace(path, serde_json::to_vec_pretty(&complete)?.as_slice())?;
    Ok(())
}
