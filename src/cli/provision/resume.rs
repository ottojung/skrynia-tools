use super::{Arguments, Result, SkryniaClient, Value, inspect, issue, secrets};
use std::path::Path;

pub fn resume(args: &Arguments) -> Result<()> {
    let path = Path::new(args.required("credentials-file")?);
    let _lock = secrets::lock(path)?;
    let text = secrets::read(
        path.to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid credential path"))?,
    )?;
    let record: Value = serde_json::from_str(&text)?;
    let state = inspect::summary(&record)?;
    let mode = state["state"].as_str().unwrap_or("");
    match mode {
        "ready" => {
            println!("{}", serde_json::to_string_pretty(&state)?);
            return Ok(());
        }
        "pending_grant_create" => anyhow::bail!(
            "grant outcome unknown; issuer token unavailable, cannot safely replay grant creation"
        ),
        "grant_saved" | "issuing_user" | "legacy_grant_saved" => {}
        _ => anyhow::bail!("credential file cannot be resumed"),
    }
    if state["requiresDuplicateRiskAcknowledgement"] == true
        && args.optional("accept-duplicate-token-risk") != Some("true")
    {
        anyhow::bail!(
            "user-token issuance may have committed; require explicit             --accept-duplicate-token-risk true to reissue"
        );
    }
    let grant = if mode == "legacy_grant_saved" {
        &record
    } else {
        &record["grant"]
    };
    issue::issue(&SkryniaClient::new(args.required("url")?)?, path, grant)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&inspect::summary(&serde_json::from_str::<Value>(
            &secrets::read(path.to_str().unwrap())?
        )?)?)?
    );
    Ok(())
}
