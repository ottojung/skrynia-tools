//! Root-overlap validation. Public and private roots must not be the same
//! directory and must not contain one another in either direction.

use super::identity::same;
use std::os::fd::OwnedFd;
use std::path::Path;

pub(crate) fn check(
    requests: &OwnedFd,
    disk: &OwnedFd,
    nginx: &OwnedFd,
    req_abs: &Path,
    disk_abs: &Path,
    nginx_abs: &Path,
) -> Result<(), String> {
    if same(requests, disk) || same(requests, nginx) || same(disk, nginx) {
        return Err("roots must be distinct directories".into());
    }
    overlap(nginx_abs, req_abs, "requests")?;
    overlap(nginx_abs, disk_abs, "disk")
}

fn overlap(nginx: &Path, private: &Path, name: &str) -> Result<(), String> {
    if nginx.starts_with(private) || private.starts_with(nginx) {
        return Err(format!(
            "{name} root and nginx root must not overlap: {} vs {}",
            private.display(),
            nginx.display()
        ));
    }
    Ok(())
}
