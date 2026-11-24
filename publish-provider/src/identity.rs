//! Device/inode identity check so roots cannot alias each other.

use std::os::fd::{AsRawFd, OwnedFd};

pub(crate) fn same(a: &OwnedFd, b: &OwnedFd) -> bool {
    let mut sa: libc::stat = unsafe { std::mem::zeroed() };
    let mut sb: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(a.as_raw_fd(), &mut sa) } != 0
        || unsafe { libc::fstat(b.as_raw_fd(), &mut sb) } != 0
    {
        return false;
    }
    sa.st_dev == sb.st_dev && sa.st_ino == sb.st_ino
}
