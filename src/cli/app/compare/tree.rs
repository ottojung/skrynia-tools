use super::{hash, types::Artifact};
use anyhow::{Result, bail};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{collections::BTreeMap, fs, path::Path};

pub fn scan(root: &Path) -> Result<BTreeMap<String, Artifact>> {
    if !fs::symlink_metadata(root)?.file_type().is_dir() {
        bail!("release artifact root must be a real directory");
    }
    let mut files = BTreeMap::new();
    visit(root, Path::new(""), &mut files)?;
    Ok(files)
}
fn visit(root: &Path, relative: &Path, out: &mut BTreeMap<String, Artifact>) -> Result<()> {
    let path = root.join(relative);
    if !fs::symlink_metadata(&path)?.file_type().is_dir() {
        bail!("artifact directory changed");
    }
    for child in fs::read_dir(path)? {
        let child = child?;
        let relative = relative.join(child.file_name());
        let name = relative
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-UTF8 release path"))?;
        let kind = child.file_type()?;
        let artifact = if kind.is_dir() {
            visit(root, &relative, out)?;
            Artifact {
                kind: "directory",
                sha256: None,
                executable: None,
            }
        } else if kind.is_file() {
            let meta = child.metadata()?;
            #[cfg(unix)]
            let executable = meta.permissions().mode() & 0o111 != 0;
            #[cfg(not(unix))]
            let executable = false;
            Artifact {
                kind: "file",
                sha256: Some(hash::sha256(&child.path(), meta.len())?),
                executable: Some(executable),
            }
        } else {
            bail!("release has symlink or special file: {name:?}");
        };
        out.insert(name.to_string(), artifact);
    }
    Ok(())
}
