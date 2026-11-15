use super::{Arguments, Result, Value, secrets};
use std::path::Path;

pub fn summary(value: &Value) -> Result<Value> {
    let (state, grant, needs_ack) = match value.get("state").and_then(Value::as_str) {
        Some("pending_grant_create") => ("pending_grant_create", None, false),
        Some("grant_saved") => ("grant_saved", value.get("grant"), false),
        Some("issuing_user") => ("issuing_user", value.get("grant"), true),
        Some(_) => anyhow::bail!("unknown credential state"),
        None if value.get("grant").is_some() && value.get("user").is_some() => {
            ("ready", value.get("grant"), false)
        }
        None if value.get("issuerToken").is_some() => ("legacy_grant_saved", Some(value), true),
        _ => anyhow::bail!("unrecognized credential record"),
    };
    Ok(serde_json::json!({"state":state,
        "grantId":grant.and_then(|g|g.get("id")).and_then(Value::as_str),
        "requiresDuplicateRiskAcknowledgement":needs_ack}))
}
pub fn inspect(args: &Arguments) -> Result<()> {
    let path = Path::new(args.required("credentials-file")?);
    let data: Value = serde_json::from_str(&secrets::read(
        path.to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid credential path"))?,
    )?)?;
    println!("{}", serde_json::to_string_pretty(&summary(&data)?)?);
    Ok(())
}
