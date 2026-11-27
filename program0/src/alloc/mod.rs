//! Software shared-RAM allocator: one monotone arena, one CAS byte per slot
//! plus one process-wide resource-lease byte. No global mutex, native lock or
//! execution budget: all coordination is single-byte CAS on shared RAM.
use super::abi;

mod lease;
mod slot;

pub use lease::*;
pub use slot::*;

pub const ERR_RANGE: i32 = -6;
pub const FREE: i32 = 0;
pub const BUSY: i32 = 1;
pub const SLOTS: u64 = 16;
pub const SLOT_SIZE: u64 = 2 * 1024 * 1024;
pub const BASE: u64 = 4096;
pub const ARENA: u64 = SLOTS * SLOT_SIZE;
pub const META_AT: u64 = 64;
pub const SLOT_PATH_AT: u64 = 128;
pub const SLOT_CODE_AT: u64 = 1152;
/// One absolute, slot-independent byte: the generic userland lease owner.
pub const LOCK_AT: u64 = 80;
/// Non-owner or unheld release attempt, reported as a precise protocol error.
pub const ERR_NOT_OWNER: i32 = -4;

/// Single-byte CAS primitive shared by slot claims and the lease protocol.
pub(super) fn cas(fd: i32, offset: u64, expected: i32, desired: i32) -> i32 {
    unsafe { abi::cas1(fd, offset, expected, desired) }
}
