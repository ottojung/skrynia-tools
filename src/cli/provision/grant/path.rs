use super::Arguments;
use anyhow::{Context, Result, bail};
use std::{fs, path::PathBuf};
pub fn destination(args: &Arguments) -> Result<PathBuf> {
    if let Some(path) = args.optional("credentials-out") {
        if path.is_empty() {
            bail!("--credentials-out cannot be empty");
        }
        return Ok(PathBuf::from(path));
    }
    let state = match std::env::var_os("XDG_STATE_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            PathBuf::from(std::env::var_os("HOME").context("HOME not set; use --credentials-out")?)
                .join(".local/state")
        }
    };
    let root = state.join("skrynia-tools/grants");
    fs::create_dir_all(&root)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    }
    if !fs::symlink_metadata(&root)?.file_type().is_dir() {
        bail!("unsafe credential directory");
    }
    Ok(root.join(format!("{:032x}.json", rand::random::<u128>())))
}
