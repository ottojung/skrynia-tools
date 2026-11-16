use super::{Result, SkryniaClient, Value, secrets};
use std::path::Path;
pub fn run(
    client: &SkryniaClient,
    file: &Path,
    record: &Value,
    mode: &str,
    id: &str,
) -> Result<()> {
    if mode == "grant_revoked" {
        return Ok(());
    }
    if !matches!(
        mode,
        "ready"
            | "grant_saved"
            | "issuing_user"
            | "revoking_user"
            | "user_revoked"
            | "revoking_grant"
    ) {
        anyhow::bail!("grant issuer credential is unavailable for revocation");
    }
    let grant = &record["grant"];
    let issuer = grant["issuerToken"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("issuer token missing"))?;
    if mode != "revoking_grant" {
        let pending = serde_json::json!({"state":"revoking_grant","grant":grant});
        secrets::replace(file, &serde_json::to_vec_pretty(&pending)?)?;
    }
    client.revoke_grant(id, issuer)?;
    let done = serde_json::json!({"state":"grant_revoked","grantId":id});
    secrets::replace(file, &serde_json::to_vec_pretty(&done)?)?;
    Ok(())
}
