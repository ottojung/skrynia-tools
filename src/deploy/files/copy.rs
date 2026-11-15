use super::{sync_dir, sync_file};
use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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
            {
                let source_mode = fs::metadata(child.path())?.permissions().mode();
                let normalized = if source_mode & 0o111 != 0 {
                    0o755
                } else {
                    0o644
                };
                fs::set_permissions(&target, fs::Permissions::from_mode(normalized))?;
            }
            sync_file(&target)?;
        } else {
            bail!(
                "build contains a symlink or special file: {}",
                child.path().display()
            );
        }
    }
    sync_dir(dst)
}
