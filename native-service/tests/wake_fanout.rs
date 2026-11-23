//! Deterministic fan-out proof with the fixed wake slot over real P1 source.
mod common;
use common::*;

#[test]
fn gap_mutating_b_never_wakes_subscriber_of_a() {
    let d = Mem::default();
    subscribe(&d, b"traveller-tes-v1");
    put(&d, b"thread-0042-tes-v1", b"msg");
    assert_eq!(hints(&d, b"traveller-tes-v1"), 0, "no wake for A");
    assert_eq!(hints(&d, b"thread-0042-tes-v1"), 0, "B has no subscriber");
    assert!(data_exists(&d, b"thread-0042-tes-v1"));
}

#[test]
fn wake_object_is_invalidated_for_every_mutation() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    for key in [
        b"traveller-tes-v1" as &[u8],
        b"traveller-vitalik-v1",
        b"thread-0113-vitalik-v1",
    ] {
        put(&d, key, b"v");
    }
    assert_eq!(list(&d), vec![3], "one slot advances per mutation");
    assert_eq!(hints(&d, WAKE), 0, "wake slot creates no hint files");
}

#[test]
fn ordinary_data_key_hint_queue_is_unchanged() {
    let d = Mem::default();
    subscribe(&d, b"data-key-v1");
    put(&d, b"data-key-v1", b"v");
    assert_eq!(hints(&d, b"data-key-v1"), 1, "data-key hints still queued");
    assert_eq!(hints(&d, WAKE), 0, "no wake slot written without a sub");
}
