//! Deterministic RED/GREEN proof for exact-key wake fan-out.
//! Real P1 appdata source (put/subscribe/hint queue) over an in-memory Disk.
//! A: gap proof (passes on base). B: desired wake behaviour (RED on base).
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
    for key in [
        b"traveller-tes-v1" as &[u8],
        b"traveller-vitalik-v1",
        b"thread-0113-vitalik-v1",
    ] {
        put(&d, key, b"v");
    }
    assert!(
        hints(&d, WAKE) >= 3,
        "wake object must fan out all mutations"
    );
}
