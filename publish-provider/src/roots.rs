//! Three disjoint, operator-owned roots opened once as confined directory
//! descriptors. The source root is never served; only symlinks are published.

use super::identity::same;
use std::ffi::CString;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct Roots {
    pub requests: OwnedFd,
    pub disk: OwnedFd,
    pub nginx: OwnedFd,
    pub disk_abs: PathBuf,
}

pub fn open(requests: &Path, disk: &Path, nginx: &Path) -> Result<Roots, String> {
    let (requests, req_abs) = open_dir(requests)?;
    let (disk, disk_abs) = open_dir(disk)?;
    let (nginx, nginx_abs) = open_dir(nginx)?;
    if same(&requests, &disk) || same(&requests, &nginx) || same(&disk, &nginx) {
        return Err("roots must be distinct directories".into());
    }
    if nginx_abs.starts_with(&req_abs) || nginx_abs.starts_with(&disk_abs) {
        return Err("private roots must not live under the nginx root".into());
    }
    Ok(Roots {
        requests,
        disk,
        nginx,
        disk_abs,
    })
}

fn open_dir(path: &Path) -> Result<(OwnedFd, PathBuf), String> {
    let abs = std::fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve {}: {e}", path.display()))?;
    let name = CString::new(abs.as_os_str().as_bytes()).map_err(|_| "invalid root path")?;
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let raw = unsafe { libc::open(name.as_ptr(), flags) };
    if raw < 0 {
        return Err(format!(
            "cannot open {}: {}",
            abs.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok((unsafe { OwnedFd::from_raw_fd(raw) }, abs))
}
