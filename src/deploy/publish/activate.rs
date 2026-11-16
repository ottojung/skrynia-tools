use super::{files, inspect, process};
use anyhow::{Result, bail};
use std::path::Path;

pub fn activate(
    root: &Path,
    name: &str,
    expected: Option<&str>,
    generation: u64,
    release: &str,
) -> Result<()> {
    if files::generation::current(root, name)? != generation {
        bail!(
            "active release was switched during build; prepared release {release} remains available"
        );
    }
    let current = inspect::inspect(root, name)?.map(|r| r.id);
    if current.as_deref() != expected {
        bail!("active release changed during build; prepared release {release} remains available");
    }
    files::switch(root, name, release)
}

/// Atomically observe the active identity and generation before a long build.
pub fn observe(root: &Path, name: &str) -> Result<(Option<String>, u64)> {
    let _lock = process::lock(root, name)?;
    let current = inspect::inspect(root, name)?.map(|r| r.id);
    let epoch = files::generation::current(root, name)?;
    Ok((current, epoch))
}
