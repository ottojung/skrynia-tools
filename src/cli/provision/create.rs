use super::{Arguments, Result, SkryniaClient, issue, secrets};
use std::path::Path;

pub fn create(
    args: &Arguments,
    name: &str,
    prefix: &str,
    matcher: &str,
    ops: &[&str],
    bytes: u64,
    objects: u64,
) -> Result<()> {
    let path = Path::new(args.required("credentials-out")?);
    let _lock = secrets::lock(path)?;
    let token = secrets::read(args.required("parent-token-file")?)?;
    let client = SkryniaClient::new(args.required("url")?)?.with_token(token);
    let pending = serde_json::json!({"state":"pending_grant_create","name":name,
        "prefix":prefix,"locations":matcher,"operations":ops,
        "maxBytes":bytes,"maxObjects":objects});
    secrets::write_new(path, serde_json::to_vec_pretty(&pending)?.as_slice())?;
    let grant = client.create_grant(matcher, ops, bytes, objects)?;
    let grant_id = grant["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("grant ID missing"))?
        .to_owned();
    let saved = serde_json::json!({"state":"grant_saved","grant":grant});
    secrets::replace(path, serde_json::to_vec_pretty(&saved)?.as_slice())?;
    issue::issue(&client, path, &grant)?;
    println!(
        "{}",
        serde_json::json!({"name":name,"prefix":prefix,
        "grantId":grant_id,"credentialsFile":path.display().to_string()})
    );
    Ok(())
}
