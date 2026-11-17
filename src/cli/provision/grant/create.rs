use super::{SkryniaClient, secrets};
use anyhow::{Context, Result};
use std::path::Path;
pub fn run(
    client: &SkryniaClient,
    path: &Path,
    loc: &str,
    ops: &[String],
    bytes: u64,
    objects: u64,
) -> Result<()> {
    let pending = serde_json::json!({"state":"pending_grant_create",
        "locations":loc,"operations":ops,"maxBytes":bytes,"maxObjects":objects});
    secrets::write_new(path, &serde_json::to_vec_pretty(&pending)?)?;
    let names: Vec<_> = ops.iter().map(String::as_str).collect();
    let grant = client.create_grant(loc, &names, bytes, objects)?;
    let id = grant["id"].as_str().context("grant ID missing")?.to_owned();
    let saved = serde_json::json!({"state":"grant_saved","grant":grant});
    secrets::replace(path, &serde_json::to_vec_pretty(&saved)?)?;
    super::super::issue::issue(client, path, &grant)?;
    println!(
        "{}",
        serde_json::json!({"grantId":id,"credentialsFile":path,
        "hint":"user token is stored privately at credentialsFile as .user.userToken"})
    );
    Ok(())
}
