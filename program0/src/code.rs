//! Stage the operator-selected guest and path strings into shared RAM.
//! Only read1/write1 carry bytes; no linear-memory pointer is passed to ABI.
use super::{abi, alloc};

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

pub fn stage(fd: i32, pos: &mut u64, base: u64, path_at: u64) -> Result<u64, i32> {
    let src = unsafe { abi::open_disk(path_at, GUEST_PATH.len() as u64, 1) };
    if src < 0 {
        return Err(src);
    }
    alloc::place(fd, pos, base)?;
    let mut len = 0u64;
    let rc = loop {
        let b = unsafe { abi::read1(src) };
        if b == super::auth::EOF {
            break Ok(len);
        }
        if b < 0 {
            break Err(b);
        }
        if len >= alloc::SLOT_SIZE {
            break Err(alloc::ERR_RANGE);
        }
        let w = unsafe { abi::write1(fd, b) };
        if w != 0 {
            break Err(w);
        }
        len += 1;
        *pos += 1;
    };
    unsafe { abi::close(src) };
    rc
}
