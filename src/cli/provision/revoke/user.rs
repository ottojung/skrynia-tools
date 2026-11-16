use super::{Result, SkryniaClient, Value, secrets};
use std::path::Path;
pub fn run(
    client: &SkryniaClient,
    file: &Path,
    record: &Value,
    mode: &str,
    id: &str,
) -> Result<()> {
    if mode == "user_revoked" {
        return Ok(());
    }
    if !matches!(mode, "ready" | "revoking_user") {
        anyhow::bail!("credential journal cannot revoke an unknown user token");
    }
    let grant = &record["grant"];
    let issuer = grant["issuerToken"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("issuer token missing"))?;
    let user_id = record["user"]["userId"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("user ID missing"))?;
    if mode == "ready" {
        let pending =
            serde_json::json!({"state":"revoking_user","grant":grant,"user":record["user"]});
        secrets::replace(file, &serde_json::to_vec_pretty(&pending)?)?;
    }
    client.revoke_user(id, user_id, issuer)?;
    let done = serde_json::json!({"state":"user_revoked","grant":grant,"userId":user_id});
    secrets::replace(file, &serde_json::to_vec_pretty(&done)?)?;
    Ok(())
}
