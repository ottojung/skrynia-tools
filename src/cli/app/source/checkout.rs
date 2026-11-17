use super::Arguments;
use anyhow::{Result, bail};
use std::{env, path::PathBuf, process::Command};
pub fn clone(args: &Arguments) -> Result<PathBuf> {
    let repo = args.required("repo")?;
    let commit = args.required("commit")?;
    let Some(slug) = repo.strip_prefix("https://github.com/") else {
        bail!("--repo must be a GitHub HTTPS repository URL");
    };
    let slug = slug.strip_suffix(".git").unwrap_or(slug);
    if slug.split('/').count() != 2
        || slug.split('/').any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        })
    {
        bail!("--repo must be a GitHub HTTPS repository URL");
    }
    if commit.len() != 40 || !commit.bytes().all(|x| x.is_ascii_hexdigit()) {
        bail!("invalid Git commit");
    }
    let root = env::temp_dir().join(format!("skrynia-source-{:032x}", rand::random::<u128>()));
    let out = Command::new("git")
        .args(["clone", "--quiet", "--no-checkout", repo])
        .arg(&root)
        .output()?;
    if !out.status.success() {
        let _ = std::fs::remove_dir_all(&root);
        bail!(
            "failed to clone source: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let out = Command::new("git")
        .args(["-C"])
        .arg(&root)
        .args(["checkout", "--quiet", "--detach", commit])
        .output()?;
    if !out.status.success() {
        let _ = std::fs::remove_dir_all(&root);
        bail!(
            "failed to checkout commit: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(root)
}
