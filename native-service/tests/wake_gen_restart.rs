//! Restart/ABA gates: a durable generation outlives sender instances, and a
//! replayed or stale ACK can never clear the current pending wake.
mod common;
use common::*;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::push;

#[test]
fn generation_is_durable_for_a_fresh_sender_after_restart() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"before", b"v");
    let before = pending(&d).clone();
    seed_sender(&d, &SENDER_B); // new sender instance over the same durable Disk
    put(&d, b"after", b"v");
    let after = pending(&d);
    assert_eq!(after.len(), 1);
    assert!(after[0] > before[0], "generation survives sender restart");
    ack(&d, &SENDER_B, before[0]);
    assert_eq!(pending(&d), after, "old sender's ACK cannot clear new wake");
    ack(&d, &SENDER_B, after[0]);
    assert!(pending(&d).is_empty());
}

#[test]
fn replayed_ack_is_idempotent_and_never_removes_current() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"one", b"v");
    let g1 = *pending(&d).last().unwrap();
    put(&d, b"two", b"v");
    let g2 = *pending(&d).last().unwrap();
    d.write_atomic(&push::hint_path("d", &ns(), WAKE, g1), &[0u8; 32])
        .unwrap(); // crash-leaked stale hint file
    ack(&d, &SENDER_A, g1);
    assert_eq!(pending(&d), vec![g2], "leaked stale hint must be cleaned");
    ack(&d, &SENDER_A, g1); // replayed ACK
    ack(&d, &SENDER_A, g2);
    assert!(pending(&d).is_empty());
    ack(&d, &SENDER_A, g2); // replayed ACK after clear
    assert!(pending(&d).is_empty());
}
