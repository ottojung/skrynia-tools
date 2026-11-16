use super::{Arguments, Result, SkryniaClient, Value, inspect, secrets};
use std::path::Path;
mod grant;
mod user;
pub fn run(args: &Arguments) -> Result<()> {
    let file = Path::new(args.required("credentials-file")?);
    let _lock = secrets::lock(file)?;
    let path = file
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("invalid credential path"))?;
    let record: Value = serde_json::from_str(&secrets::read(path)?)?;
    let summary = inspect::summary(&record)?;
    let mode = summary["state"].as_str().unwrap_or("");
    let id = summary["grantId"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no known grant ID in credential journal"))?;
    if args.required("confirm-grant-id")? != id {
        anyhow::bail!("--confirm-grant-id does not match private journal");
    }
    if matches!(mode, "revoking_user" | "revoking_grant")
        && args.optional("retry-after-uncertain") != Some("true")
    {
        anyhow::bail!(
            "previous revocation result is uncertain; require --retry-after-uncertain true"
        );
    }
    let client = SkryniaClient::new(args.required("url")?)?;
    match args.action.as_str() {
        "revoke-user" => user::run(&client, file, &record, mode, id)?,
        "revoke-grant" => grant::run(&client, file, &record, mode, id)?,
        _ => anyhow::bail!("invalid revocation action"),
    }
    let current: Value = serde_json::from_str(&secrets::read(path)?)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&inspect::summary(&current)?)?
    );
    Ok(())
}
