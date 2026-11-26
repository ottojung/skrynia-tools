//! Stage the operator-selected guest and path strings into shared RAM.
//! Only read1/write1 carry bytes; no linear-memory pointer is passed to ABI.
use super::{abi, alloc, auth};
mod stage;
pub use stage::stage;

pub const GUEST_PATH: &[u8] = b"operator/programs/root.wasm";

/// Write bytes at an absolute RAM offset, never at or beyond `limit`.
/// The whole length is checked before the first byte, so a hostile frame
/// cannot partially stage and then cross into another slot's metadata.
pub fn put(fd: i32, pos: &mut u64, off: u64, bytes: &[u8], limit: u64) -> Result<(), i32> {
    let n = bytes.len() as u64;
    if off.checked_add(n).is_none_or(|end| end > limit) {
        return Err(alloc::ERR_RANGE);
    }
    alloc::place(fd, pos, off)?;
    for &b in bytes {
        let rc = unsafe { abi::write1(fd, i32::from(b)) };
        if rc != 0 {
            return Err(rc);
        }
        *pos += 1;
    }
    Ok(())
}

/// A path may never reach the code area; hostile lengths fail closed.
pub fn put_path(fd: i32, pos: &mut u64, base: u64, bytes: &[u8]) -> Result<(), i32> {
    put(
        fd,
        pos,
        base + alloc::SLOT_PATH_AT,
        bytes,
        base + alloc::SLOT_CODE_AT,
    )
}

/// Nested EVAL code may never reach the next slot's CAS metadata.
pub fn put_code(fd: i32, pos: &mut u64, base: u64, bytes: &[u8]) -> Result<(), i32> {
    put(
        fd,
        pos,
        base + alloc::SLOT_CODE_AT,
        bytes,
        base + alloc::SLOT_SIZE,
    )
}
