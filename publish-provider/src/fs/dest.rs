//! Confined publication-root ancestor handling. Ancestors are opened with
//! O_NOFOLLOW; publish creates missing directories, unpublish never does.

use super::abi;
use super::dup;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

enum Step {
    Dir(OwnedFd),
    Absent,
}

pub(super) fn parent(
    root: &OwnedFd,
    parts: &[String],
    create: bool,
) -> Result<Option<(OwnedFd, CString)>, u32> {
    let mut current = Step::Dir(dup(root)?);
    for part in &parts[..parts.len() - 1] {
        let fd = match &current {
            Step::Dir(fd) => fd.as_raw_fd(),
            Step::Absent => return Ok(None),
        };
        current = child(fd, part, create)?;
    }
    let dir = match current {
        Step::Dir(fd) => fd,
        Step::Absent => return Ok(None),
    };
    let name = CString::new(parts[parts.len() - 1].as_bytes()).map_err(|_| abi::INVALID_PATH)?;
    Ok(Some((dir, name)))
}

fn child(parent: i32, part: &str, create: bool) -> Result<Step, u32> {
    let name = CString::new(part.as_bytes()).map_err(|_| abi::INVALID_PATH)?;
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let mut raw = unsafe { libc::openat(parent, name.as_ptr(), flags) };
    if raw < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
        if !create {
            return Ok(Step::Absent);
        }
        unsafe { libc::mkdirat(parent, name.as_ptr(), 0o755) };
        raw = unsafe { libc::openat(parent, name.as_ptr(), flags) };
    }
    if raw < 0 {
        return Err(abi::DEST_CONFLICT);
    }
    Ok(Step::Dir(unsafe { OwnedFd::from_raw_fd(raw) }))
}
