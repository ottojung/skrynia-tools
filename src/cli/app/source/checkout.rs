use super::Arguments;
use anyhow::{Result, bail};
use std::{env, path::PathBuf, process::Command};
pub fn clone(args: &Arguments) -> Result<PathBuf> {
    let repo = args.required("repo")?;
    let commit = args.required("commit")?;
    super::validate::github(repo)?;
    if commit.len() != 40 || !commit.bytes().all(|x| x.is_ascii_hexdigit()) {
        bail!("invalid Git commit");
    }
    let root = env::temp_dir().join(format!("skrynia-source-{:032x}", rand::random::<u128>()));
    let secret = args.optional("token-env").unwrap_or("SKRYNIA_TOKEN");
    let out = Command::new("git")
        .args(["clone", "--quiet", "--no-checkout", repo])
        .arg(&root)
        .env_remove("SKRYNIA_TOKEN")
        .env_remove(secret)
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
        .env_remove("SKRYNIA_TOKEN")
        .env_remove(secret)
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
