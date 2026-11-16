use super::{process, types::DeploySpec};
use anyhow::{Result, bail};
use std::path::Path;
use std::process::Command;

pub fn checkout(spec: &DeploySpec, work: &Path) -> Result<String> {
    let source = work.join("source");
    let mut clone = Command::new("git");
    clone
        .args(["clone", "--no-checkout", "--", &spec.repo])
        .arg(&source);
    process::run(clone, 180)?;
    if let Some(commit) = &spec.commit {
        let mut fetch = Command::new("git");
        fetch
            .current_dir(&source)
            .args(["fetch", "--depth", "1", "origin", commit]);
        process::run(fetch, 180)?;
    }
    let reference = spec.commit.as_deref().unwrap_or("HEAD");
    let mut checkout = Command::new("git");
    checkout
        .current_dir(&source)
        .args(["checkout", "--detach", reference]);
    process::run(checkout, 180)?;
    let mut rev_parse = Command::new("git");
    rev_parse.current_dir(&source).args(["rev-parse", "HEAD"]);
    let resolved = String::from_utf8(process::capture(rev_parse, 15)?)?
        .trim()
        .to_string();
    if !resolved.bytes().all(|c| c.is_ascii_hexdigit()) || resolved.len() != 40 {
        bail!("unexpected Git commit identity");
    }
    let root = source.canonicalize()?;
    let app = source.join(&spec.subdir).canonicalize()?;
    if !app.starts_with(root) || !app.is_dir() {
        bail!("subdirectory escapes repository");
    }
    Ok(resolved)
}
