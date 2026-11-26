//! Process-wide userland resource lease encoded in one shared RAM byte.
//!
//! The byte at `LOCK_AT` is absolute, not slot-relative: all root requests
//! contend on the same cell. `0` is free; `1..=SLOTS` is the owning slot plus
//! one. A non-owner cannot clear another request's lease, and the root exits
//! the lease on every path, including a trapped child, so no lease is orphaned.
//! This is a generic resource lease, not application policy or a kernel lock.
use super::{ERR_NOT_OWNER, LOCK_AT, cas};

/// Owner tag for a slot. Zero is reserved for free, so every owner is nonzero.
pub fn owner(slot: u64) -> i32 {
    (slot + 1) as i32
}

/// Try to take the lease for `slot`. Returns the observed byte: `0` means the
/// lease was acquired, nonzero is the value of the slot that already owns it.
pub fn lease(fd: i32, slot: u64) -> i32 {
    cas(fd, LOCK_AT, 0, owner(slot))
}

/// Release only when `slot` owns the lease. A different owner's value is left
/// untouched; the observed byte identifies whoever actually owns the cell.
pub fn unlease(fd: i32, slot: u64) -> i32 {
    cas(fd, LOCK_AT, owner(slot), 0)
}

/// Fail-safe request-exit cleanup. Idempotent and owner-only, so it recovers a
/// trapped owner's lease without stealing a lease held by another request.
pub fn reclaim(fd: i32, slot: u64) {
    unlease(fd, slot);
}

/// Relay the framed acquire/release opcode for a request's slot, mapping the
/// observed CAS byte to the P1 boolean convention: 1 acquired, 0 contended,
/// 0 released, and a negative error for a non-owner release or CAS failure.
pub fn mediated(fd: i32, base: u64, op: u8) -> i32 {
    let slot = super::slot_of(base);
    let seen = if op == 12 {
        lease(fd, slot)
    } else {
        unlease(fd, slot)
    };
    if seen < 0 {
        return seen;
    }
    if op == 12 {
        i32::from(seen == 0)
    } else if seen == owner(slot) {
        0
    } else {
        ERR_NOT_OWNER
    }
}
