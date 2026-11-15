use super::{files, release};
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct Verification {
    pub name: String,
    pub release: String,
    pub expected_sha256: Option<String>,
    pub actual_sha256: String,
    /// None means a legacy snapshot without a stored integrity commitment.
    pub verified: Option<bool>,
}
pub fn verify(root: &Path, name: &str, id: &str) -> Result<Verification> {
    let entry = release(root, name, id)?.ok_or_else(|| anyhow!("release is not available"))?;
    let tree = root.join("releases").join(name).join(id);
    let actual = files::digest_tree(&tree)?;
    let verdict = entry.content_sha256.as_ref().map(|value| *value == actual);
    Ok(Verification {
        name: name.into(),
        release: id.into(),
        expected_sha256: entry.content_sha256,
        actual_sha256: actual,
        verified: verdict,
    })
}
