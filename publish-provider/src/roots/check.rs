//! Overlap checks: public and private roots must be disjoint in both
//! directions so nginx cannot serve private material through a prefix.

use std::os::fd::{AsRawFd, OwnedFd};
use std::path::Path;

pub(super) fn roots(
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

fn same(a: &OwnedFd, b: &OwnedFd) -> bool {
    let mut sa: libc::stat = unsafe { std::mem::zeroed() };
    let mut sb: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(a.as_raw_fd(), &mut sa) } != 0
        || unsafe { libc::fstat(b.as_raw_fd(), &mut sb) } != 0
    {
        return false;
    }
    sa.st_dev == sb.st_dev && sa.st_ino == sb.st_ino
}
