//! Inspect the final node with AT_SYMLINK_NOFOLLOW. Only a symlink whose
//! bytes equal the expected absolute source path is accepted.

use super::abi;
use super::path;
use std::ffi::CStr;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub(super) enum Found {
    None,
    Same,
    Other,
}

pub(super) fn inspect(dir: &OwnedFd, name: &CStr, target: &Path) -> Result<Found, u32> {
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    let rc = unsafe {
        libc::fstatat(
            dir.as_raw_fd(),
            name.as_ptr(),
            &mut stat,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if rc != 0 {
        return match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ENOENT) => Ok(Found::None),
            _ => Err(abi::IO_ERROR),
        };
    }
    if stat.st_mode & libc::S_IFMT != libc::S_IFLNK {
        return Ok(Found::Other);
    }
    let expected = target.as_os_str().as_bytes();
    let mut buf = [0u8; path::MAX + 1];
    let n = unsafe {
        libc::readlinkat(
            dir.as_raw_fd(),
            name.as_ptr(),
            buf.as_mut_ptr().cast(),
            buf.len(),
        )
    };
    if n < 0 {
        return Err(abi::IO_ERROR);
    }
    let n = n as usize;
    if n > path::MAX || n != expected.len() || &buf[..n] != expected {
        return Ok(Found::Other);
    }
    Ok(Found::Same)
}
