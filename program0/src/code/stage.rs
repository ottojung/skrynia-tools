//! Byte-only guest staging into one exclusive shared-RAM slot.
use super::{abi, alloc, auth};

/// Stage a guest inside one slot, capped before the next slot boundary.
/// The source fd is closed on every path, including a failed relative seek.
pub fn stage(fd: i32, pos: &mut u64, off: u64, path_at: u64) -> Result<u64, i32> {
    let src = unsafe { abi::open_disk(path_at, super::GUEST_PATH.len() as u64, 1) };
    if src < 0 {
        return Err(src);
    }
    let mut len = 0u64;
    let mut rc: Result<u64, i32> = alloc::place(fd, pos, off).map(|()| 0);
    while rc.is_ok() {
        let b = unsafe { abi::read1(src) };
        if b == auth::EOF {
            rc = Ok(len);
            break;
        }
        if b < 0 {
            rc = Err(b);
            break;
        }
        if len >= alloc::SLOT_SIZE - alloc::SLOT_CODE_AT {
            rc = Err(alloc::ERR_RANGE);
            break;
        }
        let w = unsafe { abi::write1(fd, b) };
        if w != 0 {
            rc = Err(w);
            break;
        }
        len += 1;
        *pos += 1;
    }
    unsafe { abi::close(src) };
    rc
}
