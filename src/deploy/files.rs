use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::{fs, path::Path};

pub fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    if !fs::symlink_metadata(src)?.file_type().is_dir() {
        bail!("build root must be a real directory");
    }
    fs::create_dir(dst)?;
    #[cfg(unix)]
    fs::set_permissions(dst, fs::Permissions::from_mode(0o755))?;
    for child in fs::read_dir(src)? {
        let child = child?;
        let kind = fs::symlink_metadata(child.path())?.file_type();
        let target = dst.join(child.file_name());
        if kind.is_dir() {
            copy_tree(&child.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(child.path(), &target)?;
            #[cfg(unix)]
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644))?;
        } else {
            bail!(
                "build contains a symlink or special file: {}",
                child.path().display()
            );
        }
    }
    Ok(())
}
pub fn switch(root: &Path, name: &str, release: &str) -> Result<()> {
    let link = root.join("apps").join(name);
    if fs::symlink_metadata(&link).is_ok() && !fs::symlink_metadata(&link)?.file_type().is_symlink()
    {
        bail!("application path is not an owned release symlink");
    }
    let candidate = root
        .join("apps")
        .join(format!(".{name}.{:016x}", rand::random::<u64>()));
    symlink(format!("../releases/{name}/{release}"), &candidate)?;
    if let Err(error) = fs::rename(&candidate, &link) {
        let _ = fs::remove_file(candidate);
        return Err(error.into());
    }
    Ok(())
}
