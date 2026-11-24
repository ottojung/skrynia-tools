//! Mutation of the final publication node. A racing creation or removal is
//! reconciled against the exact target, never against a bare name.

use super::abi;
use std::ffi::{CStr, CString};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub(super) fn create(dir: &OwnedFd, name: &CStr, target: &Path) -> u32 {
    let value = match CString::new(target.as_os_str().as_bytes()) {
        Ok(value) => value,
        Err(_) => return abi::INVALID_PATH,
    };
    let rc = unsafe { libc::symlinkat(value.as_ptr(), dir.as_raw_fd(), name.as_ptr()) };
    if rc == 0 { abi::OK } else { abi::DEST_CONFLICT }
}

pub(super) fn remove(dir: &OwnedFd, name: &CStr) -> u32 {
    let rc = unsafe { libc::unlinkat(dir.as_raw_fd(), name.as_ptr(), 0) };
    if rc == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
        abi::OK
    } else {
        abi::IO_ERROR
    }
}
