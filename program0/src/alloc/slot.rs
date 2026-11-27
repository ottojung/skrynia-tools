//! Monotone metadata initialization and per-request owner tags.
use super::{BUSY, ERR_RANGE, FREE, META_AT, META_END, SLOTS, abi};

pub fn ensure(fd: i32) -> Result<(), i32> {
    let target = META_END as i64;
    let at = unsafe { abi::seek(fd, target) };
    if at == target {
        return if unsafe { abi::seek(fd, -target) } == 0 {
            Ok(())
        } else {
            Err(ERR_RANGE)
        };
    }
    if at > 0 && unsafe { abi::seek(fd, -at) } != 0 {
        return Err(ERR_RANGE);
    }
    if unsafe { abi::truncate(fd, META_END) } != 0 {
        return Err(ERR_RANGE);
    }
    Ok(())
}

pub fn acquire(fd: i32) -> Result<u64, i32> {
    for i in 0..SLOTS {
        if super::cas(fd, META_AT + i, FREE, BUSY) == FREE {
            return Ok(i);
        }
    }
    Err(ERR_RANGE)
}

pub fn release(fd: i32, index: u64) {
    super::cas(fd, META_AT + index, BUSY, FREE);
}
