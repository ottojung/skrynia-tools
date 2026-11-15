use super::{files, inspect};
use anyhow::{Result, bail};
use std::path::Path;

pub fn activate(root: &Path, name: &str, expected: Option<&str>, release: &str) -> Result<()> {
    let current = inspect::inspect(root, name)?.map(|r| r.id);
    if current.as_deref() != expected {
        bail!("active release changed during build; prepared release {release} remains available");
    }
    files::switch(root, name, release)
}
