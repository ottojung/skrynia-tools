use super::{files, inspect, release, valid_name, validate};
use anyhow::{Result, bail};
use std::path::Path;
mod active;
mod candidates;
mod clean;
mod types;
mod workspaces;
pub use types::Audit;
mod generation;
mod metadata;
mod trees;

pub fn audit(root: &Path, name: &str) -> Result<Audit> {
    if !valid_name(name) || !files::existing_dir(root)? {
        bail!("invalid app name or deployment root missing");
    }
    let mut result = Audit::new(name);
    active::check(root, name, &mut result);
    generation::check(root, name, &mut result);
    metadata::scan(root, name, &mut result)?;
    trees::scan(root, name, &mut result)?;
    candidates::scan(root, name, &mut result)?;
    workspaces::scan(root, name, &mut result)?;
    result.finalize();
    Ok(result)
}
