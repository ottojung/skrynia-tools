//! Fixed-slot gates: no quota outage, no remove outage, no stale-ACK wake loss.
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
    assert_eq!(
        list(&d),
        vec![n as u64],
        "latest generation stays discoverable"
    );
}

#[test]
fn stale_ack_after_mutation_retains_new_wake() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, b"first", b"v");
    let g1 = list(&d)[0];
    put(&d, b"second", b"v");
    let g2 = list(&d)[0];
    assert_eq!(list(&d).len(), 1, "one fixed slot, never a backlog");
    assert!(g2 > g1, "generation must advance under mutation");
    assert!(!read_ok(&d, g1), "superseded generation is not readable");
    ack(&d, &SENDER_A, g1);
    assert_eq!(list(&d), vec![g2], "stale ACK must retain newer wake");
    assert!(read_ok(&d, g2));
    ack(&d, &SENDER_A, g2);
    assert!(list(&d).is_empty(), "current ACK clears the slot");
}

#[test]
fn remove_denial_never_blocks_writes_or_hides_latest_wake() {
    let mem = Mem::default();
    subscribe(&mem, WAKE);
    seed_sender(&mem, &SENDER_A);
    let d = NoRemove(mem);
    for i in 0..8 {
        assert!(
            put_ok(&d, format!("rm-{i}").as_bytes()),
            "remove denial must not block a data write"
        );
    }
    assert_eq!(list(&d), vec![8], "no stale file can mask the slot");
    assert!(read_ok(&d, 8));
    ack(&d, &SENDER_A, 8);
    assert!(list(&d).is_empty());
}
