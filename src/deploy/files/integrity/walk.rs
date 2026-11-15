use super::{file, record};
use anyhow::{Result, bail};
use sha2::Sha256;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

pub fn visit(root: &Path, relative: &Path, hash: &mut Sha256) -> Result<()> {
    let directory = root.join(relative);
    if !fs::symlink_metadata(&directory)?.file_type().is_dir() {
        bail!("release directory changed during hashing");
    }
    let mut children = fs::read_dir(&directory)?.collect::<std::io::Result<Vec<_>>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = relative.join(child.file_name());
        let kind = child.file_type()?;
        if kind.is_dir() {
            record::add(hash, b'D', &path, 0);
            visit(root, &path, hash)?;
        } else if kind.is_file() {
            let size = child.metadata()?.len();
            #[cfg(unix)]
            let marker = if child.metadata()?.permissions().mode() & 0o111 != 0 {
                b'X'
            } else {
                b'F'
            };
            #[cfg(not(unix))]
            let marker = b'F';
            record::add(hash, marker, &path, size);
            file::hash(&child.path(), size, hash)?;
        } else {
            bail!(
                "release contains symlink or special file: {}",
                path.display()
            );
        }
    }
    Ok(())
}
