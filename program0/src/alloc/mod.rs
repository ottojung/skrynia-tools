//! Shared RAM contains only owner tags and a process-wide resource lease.
//! Private WASM memory holds all transient request buffers.
use super::abi;

mod lease;
mod slot;

pub use lease::*;
pub use slot::*;

pub const ERR_RANGE: i32 = -6;
pub const FREE: i32 = 0;
pub const BUSY: i32 = 1;
pub const SLOTS: u64 = 16;
pub const META_AT: u64 = 64;
/// One absolute, slot-independent byte: the generic userland lease owner.
pub const LOCK_AT: u64 = META_AT + SLOTS;
pub const META_END: u64 = LOCK_AT + 1;
/// Non-owner or unheld release attempt, reported as a precise protocol error.
pub const ERR_NOT_OWNER: i32 = -4;

/// Single-byte CAS primitive shared by slot claims and the lease protocol.
pub(super) fn cas(fd: i32, offset: u64, expected: i32, desired: i32) -> i32 {
    unsafe { abi::cas1(fd, offset, expected, desired) }
}
