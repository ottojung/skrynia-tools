//! P1-wide userland lease: one shared byte, owner-tagged, fail-safe on exit.
use super::{alloc, entry, fake, invoke, seed, support};

#[test]
fn lease_byte_is_shared_and_owner_only() {
    let (_i, _o, ram) = fake::seed(vec![], vec![]);
    alloc::ensure(ram).expect("arena");
    assert_eq!(alloc::lease(ram, 0), 0);
    assert_eq!(alloc::lease(ram, 1), alloc::owner(0));
    assert_eq!(alloc::unlease(ram, 1), alloc::owner(0));
    assert_eq!(alloc::lease(ram, 0), alloc::owner(0));
    assert_eq!(alloc::unlease(ram, 0), alloc::owner(0));
    assert_eq!(alloc::lease(ram, 1), 0);
    assert_eq!(alloc::owner(5), 6);
}

#[test]
fn framed_ops_exclude_other_requests_without_releasing_their_lease() {
    let (i, o, ram) = fake::seed(vec![], vec![]);
    alloc::ensure(ram).expect("arena");
    let (_slot_a, base_a) = alloc::acquire(ram).expect("slot a");
    let (_slot_b, base_b) = alloc::acquire(ram).expect("slot b");
    invoke::enter(i, o, ram, base_a, i, o, 0);
    assert_eq!(support::code(&support::call(&[12])), 1);
    invoke::leave();
    invoke::enter(i, o, ram, base_b, i, o, 0);
    assert_eq!(support::code(&support::call(&[12])), 0);
    assert_eq!(support::code(&support::call(&[13])), alloc::ERR_NOT_OWNER);
    invoke::leave();
    invoke::enter(i, o, ram, base_a, i, o, 0);
    assert_eq!(support::code(&support::call(&[13])), 0);
    invoke::leave();
    fake::inspect(|s| assert_eq!(s.ram[alloc::LOCK_AT as usize], alloc::FREE as u8));
    assert_eq!(alloc::slot_of(base_b), 1);
}

#[test]
fn request_exit_reclaims_a_trapped_owner_lease() {
    let bearer = [0x66u8; 32];
    let (i, o, ram) = seed(&bearer, &bearer, b"guest", b"x");
    alloc::ensure(ram).expect("arena");
    assert_eq!(alloc::lease(ram, 0), 0);
    fake::set_eval_result(-1);
    assert!(entry::skrynia_main(i, o) < 0);
    fake::inspect(|s| assert_eq!(s.ram[alloc::LOCK_AT as usize], alloc::FREE as u8));
    assert_eq!(alloc::lease(ram, 0), 0);
    alloc::reclaim(ram, 0);
}
