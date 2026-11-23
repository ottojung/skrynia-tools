//! Nonvacuous gates: wake quota must never block data writes, and a stale ACK
//! must never clear a newer dirty generation. Real P1 source, in-memory Disk.
mod common;
use common::*;
use skrynia_native_service::registry::appdata::push;

#[test]
fn wake_quota_never_blocks_ordinary_data_writes() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    let n = push::MAX_HINTS + 64;
    let mut committed = 0;
    for i in 0..n {
        if put_ok(&d, format!("key-{i:05}").as_bytes()) {
            committed += 1;
        }
    }
    assert_eq!(committed, n, "no write may fail just because wake is full");
    assert!(data_exists(&d, format!("key-{:05}", n - 1).as_bytes()));
    assert_eq!(pending(&d).len(), 1, "wake state must stay coalesced");
}

#[test]
fn stale_ack_after_mutation_retains_new_wake() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"first", b"v");
    let g1 = *pending(&d).last().unwrap();
    put(&d, b"second", b"v");
    let g2 = *pending(&d).last().unwrap();
    assert_eq!(pending(&d).len(), 1, "two mutations coalesce to one wake");
    assert!(g2 > g1, "durable generation must advance");
    ack(&d, &SENDER_A, g1);
    assert_eq!(pending(&d), vec![g2], "stale ACK must retain newer wake");
    ack(&d, &SENDER_A, g2);
    assert!(pending(&d).is_empty(), "current ACK clears the wake");
}
