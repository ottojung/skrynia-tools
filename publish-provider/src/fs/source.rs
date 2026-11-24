//! Confined source lookup: never follow a symlink in any component, and
//! require the final node to be a regular file under the operator disk root.

use super::Roots;
use super::abi;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

pub fn open(roots: &Roots, parts: &[String]) -> Result<OwnedFd, u32> {
    let mut parent = roots.disk.as_raw_fd();
    let mut opened: Vec<OwnedFd> = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let name = CString::new(part.as_bytes()).map_err(|_| abi::INVALID_PATH)?;
        let last = index + 1 == parts.len();
        let mut flags = libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC;
        if !last {
            flags |= libc::O_DIRECTORY;
        }
        let raw = unsafe { libc::openat(parent, name.as_ptr(), flags) };
        if raw < 0 {
            return Err(abi::INVALID_SOURCE);
        }
        let owned = unsafe { OwnedFd::from_raw_fd(raw) };
        if last {
            let file = std::fs::File::from(owned);
            let meta = file.metadata().map_err(|_| abi::IO_ERROR)?;
            if !meta.is_file() {
                return Err(abi::INVALID_SOURCE);
            }
            // Keep regular file open to pin existence, but O_NONBLOCK is harmless.
            return Ok(file.into());
        }
        opened.push(owned);
        parent = opened.last().ok_or(abi::INTERNAL)?.as_raw_fd();
    }
    Err(abi::INVALID_PATH)
}
