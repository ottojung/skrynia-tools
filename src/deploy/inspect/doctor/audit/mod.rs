use super::{files, inspect, release, valid_name, validate};
use anyhow::{Result, bail};
use serde::Serialize;
use std::path::Path;
mod clean;
mod metadata;
mod trees;

#[derive(Debug, Serialize)]
pub struct Audit {
    pub name: String,
    pub available_releases: usize,
    pub active_release: Option<String>,
    pub active_error: Option<String>,
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
        active_release: None,
        active_error: None,
        metadata_without_artifact: vec![],
        artifact_without_metadata: vec![],
        abandoned_staging: vec![],
        invalid_entries: vec![],
        clean: false,
    };
    match inspect(root, name) {
        Ok(Some(active)) => result.active_release = Some(active.id),
        Ok(None) => {}
        Err(error) => result.active_error = Some(error.to_string()),
    }
    metadata::scan(root, name, &mut result)?;
    trees::scan(root, name, &mut result)?;
    result.finalize();
    Ok(result)
}
