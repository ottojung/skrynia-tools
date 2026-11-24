//! Operator-private request mailbox: eight lowercase hex digits name a file
//! holding the exact UTF-8 relative path bytes (no newline, at most 4096).

use super::abi;
use super::path;
use std::ffi::CString;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

/// File name for a ticket, fixed width so concurrent writers cannot collide.
pub fn name(ticket: u32) -> String {
    format!("{ticket:08x}")
}

pub fn read(root: &OwnedFd, ticket: u32) -> Result<Vec<u8>, u32> {
    let name = CString::new(name(ticket).into_bytes()).map_err(|_| abi::BAD_TICKET)?;
    let flags = libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let raw = unsafe { libc::openat(root.as_raw_fd(), name.as_ptr(), flags) };
    if raw < 0 {
        return Err(abi::BAD_TICKET);
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(raw) };
    let mut bytes = Vec::new();
    file.by_ref()
        .take((path::MAX + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| abi::IO_ERROR)?;
    if bytes.len() > path::MAX {
        return Err(abi::INVALID_PATH);
    }
    Ok(bytes)
}
