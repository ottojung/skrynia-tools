//! Allocator and T3-agreed invoke ops; no native lock or host budget.
use super::{alloc, fake, invoke};

#[test]
fn slots_are_cas_isolated_and_reused_after_release() {
    let (_i, _o, ram) = fake::seed(vec![], vec![]);
    alloc::ensure(ram).expect("arena");
    let a = alloc::acquire(ram).expect("first slot");
    let b = alloc::acquire(ram).expect("second slot");
    assert_ne!(a, b);
    alloc::release(ram, a);
    assert_eq!(a, alloc::acquire(ram).expect("reused"));
}

#[test]
fn get_put_flush_and_reply_eof_are_request_scoped() {
    let (input, output, ram) = fake::seed(vec![7, 8, 9], vec![]);
    alloc::ensure(ram).expect("arena");
    let slot = alloc::acquire(ram).expect("slot");
    invoke::enter(input, output, ram, slot, input, output);
    assert_eq!(invoke::invoke(0, invoke::GET, 0), 7);
    assert_eq!(invoke::invoke(0, invoke::GET, 0), 8);
    for b in [3u8, 1, 0, 0, 0, 1, 0, 0, 0, 42] {
        assert_eq!(invoke::invoke(0, invoke::PUT, i32::from(b)), 0);
    }
    assert_eq!(invoke::invoke(0, invoke::FLUSH, 0), 0);
    assert_eq!(invoke::invoke(0, invoke::GET, 0), 1);
    assert_eq!(invoke::invoke(0, invoke::PUT, 9), -3);
    assert_eq!(invoke::invoke(0, invoke::FLUSH, 0), -3);
    for _ in 0..3 {
        assert_eq!(invoke::invoke(0, invoke::GET, 0), 0);
    }
    assert_eq!(invoke::invoke(0, invoke::GET, 0), -1);
    assert_eq!(invoke::invoke(0, invoke::GET, 0), 9);
    assert_eq!(invoke::invoke(0, invoke::CB_PUT, 9), -4);
    assert_eq!(invoke::invoke(0, invoke::CB_GET, 0), -4);
    assert_eq!(invoke::invoke(0, invoke::STATUS, 0), 0);
    assert_eq!(invoke::invoke(0, 99, 0), -3);
    invoke::leave();
    assert_eq!(invoke::invoke(0, invoke::GET, 0), -2);
    assert_eq!(invoke::invoke(0, invoke::FLUSH, 0), -2);
    fake::inspect(|s| assert_eq!(s.output, vec![42]));
}
