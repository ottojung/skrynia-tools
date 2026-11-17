use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{fs, path::Path};
pub fn find(data: &Path) -> Result<(String, String)> {
    let dir = data.join("state/grants");
    if !fs::symlink_metadata(&dir)?.file_type().is_dir() {
        bail!("Skrynia grants path is not a real directory");
    }
    let mut root = None;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|x| x.to_str()) != Some("json") {
            bail!("unexpected item in grants directory");
        }
        if !entry.file_type()?.is_file() {
            bail!("unsafe grant record");
        }
        let record: Value = serde_json::from_slice(&fs::read(entry.path())?)
            .context("unreadable or invalid Skrynia grant record")?;
        if record.get("parent").is_some_and(Value::is_null) {
            if root.is_some() {
                bail!("ambiguous Skrynia root grants");
            }
            let id = record["id"].as_str().context("root grant ID missing")?;
            let issuer = record["rootIssuerToken"]
                .as_str()
                .context("root issuer missing")?;
            if !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || issuer.len() != 64
                || !issuer.bytes().all(|b| b.is_ascii_hexdigit())
            {
                bail!("invalid Skrynia root record");
            }
            root = Some((id.to_owned(), issuer.to_owned()));
        }
    }
    root.context("Skrynia root grant not found in data directory")
}
