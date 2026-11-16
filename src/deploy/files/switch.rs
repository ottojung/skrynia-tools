use super::{generation, sync_dir};
use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::{fs, path::Path};

pub fn switch(root: &Path, name: &str, release: &str) -> Result<()> {
    let public = root.join("apps");
    let link = public.join(name);
    if fs::symlink_metadata(&link).is_ok() && !fs::symlink_metadata(&link)?.file_type().is_symlink()
    {
        bail!("application path is not an owned release symlink");
    }
    // Persist this intent before touching the active link: even A -> B -> A
    // invalidates an in-flight build that started before the administrator's change.
    generation::advance(root, name)?;
    let candidate = public.join(format!(".{name}.{:032x}", rand::random::<u128>()));
    symlink(format!("../releases/{name}/{release}"), &candidate)?;
    if let Err(error) = fs::rename(&candidate, &link) {
        let _ = fs::remove_file(candidate);
        return Err(error.into());
    }
    sync_dir(&public).map_err(|error| anyhow::anyhow!(
        "active symlink changed but durability is uncertain; inspect current before retrying: {error}"
    ))
}
