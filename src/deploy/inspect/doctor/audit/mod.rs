use super::{files, release, valid_name, validate};
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::Path;
mod metadata;
mod trees;

#[derive(Debug, Serialize)]
pub struct Audit {
    pub name: String,
    pub available_releases: usize,
    pub metadata_without_artifact: Vec<String>,
    pub artifact_without_metadata: Vec<String>,
    pub abandoned_staging: Vec<String>,
    pub invalid_entries: Vec<String>,
    pub clean: bool,
}
pub fn audit(root: &Path, name: &str) -> Result<Audit> {
    if !valid_name(name) || !files::existing_dir(root)? {
        bail!("invalid app name or deployment root missing");
    }
    let mut result = Audit {
        name: name.into(),
        available_releases: 0,
        metadata_without_artifact: vec![],
        artifact_without_metadata: vec![],
        abandoned_staging: vec![],
        invalid_entries: vec![],
        clean: false,
    };
    metadata::scan(root, name, &mut result)?;
    trees::scan(root, name, &mut result)?;
    result.metadata_without_artifact.sort();
    result.artifact_without_metadata.sort();
    result.abandoned_staging.sort();
    result.invalid_entries.sort();
    result.clean = result.metadata_without_artifact.is_empty()
        && result.artifact_without_metadata.is_empty()
        && result.abandoned_staging.is_empty()
        && result.invalid_entries.is_empty();
    Ok(result)
}
