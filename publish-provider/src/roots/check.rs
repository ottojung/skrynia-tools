//! Overlap checks: requests/mailbox, source disk and nginx roots must be
//! disjoint in both directions so one root cannot expose another through a prefix.

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
    overlap(nginx_abs, disk_abs, "disk")?;
    // A nested mailbox under the source root (or a source nested under the
    // mailbox) can be published by a generic relative path. Reject both
    // directions, not just nginx-vs-private overlaps.
    overlap(disk_abs, req_abs, "requests")
}

fn overlap(a: &Path, b: &Path, name: &str) -> Result<(), String> {
    if a.starts_with(b) || b.starts_with(a) {
        return Err(format!(
            "{name} root must not overlap the compared root: {} vs {}",
            a.display(),
            b.display()
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
