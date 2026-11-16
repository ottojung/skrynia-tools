use super::sync_dir;
use anyhow::Result;
use std::path::Path;
mod read;
mod write;
pub fn current(root: &Path, name: &str) -> Result<u64> {
    read::current(root, name)
}
pub fn advance(root: &Path, name: &str) -> Result<u64> {
    let next = current(root, name)?
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("activation generation overflow"))?;
    write::persist(root, name, next)?;
    Ok(next)
}
