//! Cursor unit tests, split from `ram` to satisfy the structural LOC limit.
use super::ram::{BASE, Cursor, ERR_BOUNDS, ERR_RANGE, SLOT};

#[test]
fn distinct_slots_are_disjoint() {
    let mut a = Cursor::for_slot(0);
    let mut b = Cursor::for_slot(1);
    assert_eq!(a.base(), BASE);
    assert_eq!(b.base(), BASE + SLOT);
    assert_eq!(a.alloc(8).unwrap(), BASE);
    assert_eq!(b.alloc(8).unwrap(), BASE + SLOT);
    assert_eq!(a.alloc(u64::MAX), Err(ERR_RANGE));
    assert_eq!(a.alloc(SLOT), Err(ERR_BOUNDS));
}

#[test]
fn reset_reclaims_only_its_own_slot() {
    let mut c = Cursor::for_slot(3);
    assert_eq!(c.alloc(4096).unwrap(), BASE + 3 * SLOT);
    c.reset();
    assert_eq!(c.alloc(4).unwrap(), BASE + 3 * SLOT);
    assert_eq!(c.next(), BASE + 3 * SLOT + 4);
}

#[test]
fn last_slot_is_bounded_and_never_wraps_into_slot_zero() {
    let mut c = Cursor::for_slot(255);
    assert_eq!(c.base(), BASE + 255 * SLOT);
    assert_eq!(c.alloc(SLOT).unwrap(), BASE + 255 * SLOT);
    assert_eq!(c.alloc(1), Err(ERR_BOUNDS));
    c.reset();
    assert_eq!(c.alloc(1).unwrap(), BASE + 255 * SLOT);
    assert_ne!(c.base(), Cursor::for_slot(0).base());
}
