//! Negative gates: dynamic keys, restart durability, revocation.
mod common;
use common::*;

#[test]
fn fanout_covers_113_dynamic_thread_keys_with_one_subscription() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    for i in 0..113 {
        put(&d, format!("thread-{i:03}-tes-v1").as_bytes(), b"msg");
    }
    assert_eq!(hints(&d, WAKE), 1, "all 113 keys coalesce to one wake");
    assert_eq!(pending(&d), vec![113], "generation counts every mutation");
    assert_eq!(hints(&d, b"thread-000-tes-v1"), 0, "no per-key subs");
}

#[test]
fn wake_hints_survive_reopen() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    put(&d, b"traveller-vitalik-v1", b"v");
    subscribe(&d, WAKE);
    assert!(hints(&d, WAKE) >= 1, "wake hints must be durable");
}

#[test]
fn revoking_the_wake_subscription_stops_fanout() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    put(&d, b"traveller-tes-v1", b"v");
    let before = hints(&d, WAKE);
    unsubscribe(&d, WAKE);
    put(&d, b"traveller-vitalik-v1", b"v");
    assert_eq!(hints(&d, WAKE), before, "no wake after revocation");
}
