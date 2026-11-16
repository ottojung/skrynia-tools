use super::{Result, Value};
pub fn summary(value: &Value) -> Result<Value> {
    let (state, grant, risk) = match value.get("state").and_then(Value::as_str) {
        Some("pending_grant_create") => ("pending_grant_create", None, false),
        Some("grant_saved") => ("grant_saved", value.get("grant"), false),
        Some("issuing_user") => ("issuing_user", value.get("grant"), true),
        Some("revoking_user") => ("revoking_user", value.get("grant"), false),
        Some("user_revoked") => ("user_revoked", value.get("grant"), false),
        Some("revoking_grant") => ("revoking_grant", value.get("grant"), false),
        Some("grant_revoked") => ("grant_revoked", None, false),
        Some(_) => anyhow::bail!("unknown credential state"),
        None if value.get("grant").is_some() && value.get("user").is_some() => {
            ("ready", value.get("grant"), false)
        }
        None if value.get("issuerToken").is_some() => ("legacy_grant_saved", Some(value), true),
        _ => anyhow::bail!("unrecognized credential record"),
    };
    let id = if state == "grant_revoked" {
        value.get("grantId")
    } else {
        grant.and_then(|v| v.get("id"))
    };
    Ok(serde_json::json!({"state":state,
        "grantId":id.and_then(Value::as_str),
        "requiresDuplicateRiskAcknowledgement":risk,
        "requiresRevocationRetryAcknowledgement":matches!(state,"revoking_user"|"revoking_grant")}))
}
