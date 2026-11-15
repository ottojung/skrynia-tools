use anyhow::{Result, bail};
use std::{env, path::Path};

pub fn enforce_root(path: &Path) -> Result<()> {
    let Some(root) = env::var_os("SKRYNIA_DEPLOY_ROOT") else {
        return Ok(());
    };
    let allowed = Path::new(&root).canonicalize()?;
    let requested = path.canonicalize()?;
    if requested != allowed {
        bail!("deployment root differs from SKRYNIA_DEPLOY_ROOT");
    }
    Ok(())
}
