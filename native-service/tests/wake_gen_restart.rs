//! Restart/ABA/re-list gates for the fixed durable wake slot.
mod common;
use common::*;

#[test]
fn generation_is_durable_for_a_fresh_sender_after_restart() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"before", b"v");
    let before = list(&d);
    seed_sender(&d, &SENDER_B); // new sender instance over the same durable Disk
    put(&d, b"after", b"v");
    let after = list(&d);
    assert_eq!(after.len(), 1);
    assert!(after[0] > before[0], "generation survives sender restart");
    ack(&d, &SENDER_B, before[0]);
    assert_eq!(list(&d), after, "old sender's ACK cannot clear new wake");
    ack(&d, &SENDER_B, after[0]);
    assert!(list(&d).is_empty());
}

#[test]
fn repeated_and_stale_acks_are_noops() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"one", b"v");
    let g1 = list(&d)[0];
    put(&d, b"two", b"v");
    let g2 = list(&d)[0];
    ack(&d, &SENDER_A, g1);
    ack(&d, &SENDER_A, g1);
    assert_eq!(list(&d), vec![g2], "replayed stale ACK cannot clear g2");
    ack(&d, &SENDER_A, g2);
    ack(&d, &SENDER_A, g2);
    assert!(list(&d).is_empty(), "replayed current ACK is idempotent");
    ack(&d, &SENDER_A, g1);
    assert!(list(&d).is_empty());
}

#[test]
fn superseded_read_is_missing_and_next_list_finds_latest() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"listed", b"v");
    let g1 = list(&d)[0];
    assert!(read_ok(&d, g1), "sender listed g1");
    put(&d, b"mutated", b"v"); // mutation between list and read
    let g2 = list(&d)[0];
    assert!(!read_ok(&d, g1), "read(g1) is Missing, not a stale wake");
    assert_eq!(list(&d), vec![g2], "next cycle re-lists the latest gen");
    assert!(read_ok(&d, g2), "no fatal sender liveness loss");
}
