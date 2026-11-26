//! Monotone arena growth and per-slot occupancy claims.
use super::{ARENA, BASE, BUSY, ERR_RANGE, FREE, META_AT, SLOT_SIZE, SLOTS, abi};

/// Grow when needed, never truncate an already larger RAM. A relative seek
/// probes capacity; reaching it proves enough room, so truncate only on growth.
/// The RAM cursor is restored to zero on every path.
pub fn ensure(fd: i32) -> Result<(), i32> {
    let target = (BASE + ARENA) as i64;
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
    if unsafe { abi::truncate(fd, target as u64) } != 0 {
        return Err(ERR_RANGE);
    }
    Ok(())
}

pub fn acquire(fd: i32) -> Result<(u64, u64), i32> {
    for i in 0..SLOTS {
        if super::cas(fd, META_AT + i, FREE, BUSY) == FREE {
            return Ok((i, BASE + i * SLOT_SIZE));
        }
    }
    Err(ERR_RANGE)
}

pub fn release(fd: i32, index: u64) {
    super::cas(fd, META_AT + index, BUSY, FREE);
}

/// Map a slot base back to its index. Every context base comes from `acquire`.
pub fn slot_of(base: u64) -> u64 {
    base.saturating_sub(BASE) / SLOT_SIZE
}

/// Move the tracked RAM cursor to an absolute offset using only relative seek.
pub fn place(fd: i32, pos: &mut u64, target: u64) -> Result<(), i32> {
    let shift = (target as i64).checked_sub(*pos as i64).ok_or(ERR_RANGE)?;
    let at = unsafe { abi::seek(fd, shift) };
    if at < 0 {
        return Err(at as i32);
    }
    *pos = at as u64;
    Ok(())
}
