//! Software shared-RAM allocator. No global mutex and no native limit is used:
//! one fixed monotone arena plus one CAS byte per slot (0 FREE, 1 BUSY).
//! Monotone truncate is safe under the K4 stable-page RAM fix.
pub const ERR_RANGE: i32 = -6;
pub const FREE: i32 = 0;
pub const BUSY: i32 = 1;
pub const SLOTS: u64 = 16;
pub const SLOT_SIZE: u64 = 2 * 1024 * 1024;
pub const BASE: u64 = 4096;
pub const ARENA: u64 = SLOTS * SLOT_SIZE;
pub const META_AT: u64 = 64;
pub const SLOT_PATH_AT: u64 = 128;
pub const SLOT_CODE_AT: u64 = 256;
pub const LOCK_AT: u64 = 192;

/// Grow once to the fixed arena before any path is staged. Growth is monotone
/// and protected by the K4 stable-page fix; no global RAM mutex is taken.
pub fn ensure(fd: i32) -> Result<(), i32> {
    if unsafe { super::abi::truncate(fd, BASE + ARENA) } != 0 {
        return Err(ERR_RANGE);
    }
    Ok(())
}

pub fn acquire(fd: i32) -> Result<(u64, u64), i32> {
    for i in 0..SLOTS {
        let seen = unsafe { super::abi::cas1(fd, META_AT + i, FREE, BUSY) };
        if seen == FREE {
            return Ok((i, BASE + i * SLOT_SIZE));
        }
    }
    Err(ERR_RANGE)
}

pub fn release(fd: i32, index: u64) {
    unsafe { super::abi::cas1(fd, META_AT + index, BUSY, FREE) };
}

/// Move the tracked RAM cursor to an absolute offset using only relative seek.
pub fn place(fd: i32, pos: &mut u64, target: u64) -> Result<(), i32> {
    let shift = (target as i64).checked_sub(*pos as i64).ok_or(ERR_RANGE)?;
    let at = unsafe { super::abi::seek(fd, shift) };
    if at < 0 {
        return Err(at as i32);
    }
    *pos = at as u64;
    Ok(())
}
