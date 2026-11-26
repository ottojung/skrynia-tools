//! Stage the operator-selected guest and path strings into shared RAM.
//! Only read1/write1 carry bytes; no linear-memory pointer is passed to ABI.
use super::{abi, alloc, auth};
mod stage;
pub use stage::stage;

pub const GUEST_PATH: &[u8] = b"programs/root.wasm";

pub fn put(fd: i32, pos: &mut u64, off: u64, bytes: &[u8]) -> Result<(), i32> {
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
pub fn put_path(fd: i32, pos: &mut u64, off: u64, bytes: &[u8]) -> Result<(), i32> {
    if bytes.len() as u64 > alloc::SLOT_CODE_AT - alloc::SLOT_PATH_AT {
        return Err(alloc::ERR_RANGE);
    }
    put(fd, pos, off, bytes)
}
