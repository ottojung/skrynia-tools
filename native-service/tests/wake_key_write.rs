//! Reserved wake key remains a valid exact data key; its direct mutation uses
//! the fixed dirty slot, never legacy per-seq hint files.
mod common;
use common::*;
use skrynia_native_service::NoLock;
use skrynia_native_service::hash_bytes;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};

#[test]
fn direct_put_to_wake_key_notifies_and_keeps_no_legacy_files() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, WAKE, b"payload");
    assert_eq!(list(&d), vec![1], "sender sees the wake-key write");
    assert!(read_ok(&d, 1));
    assert_eq!(hints(&d, WAKE), 0, "no legacy per-seq hint files");
    assert!(data_exists(&d, WAKE), "data write contract preserved");
    ack(&d, &SENDER_A, 1);
    assert!(list(&d).is_empty());
}

#[test]
fn direct_delete_of_wake_key_advances_slot_without_legacy_files() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    put(&d, WAKE, b"payload");
    let before = list(&d)[0];
    ops::delete(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        WAKE,
        &hash_bytes(b"payload"),
    )
    .unwrap();
    assert_eq!(list(&d), vec![before + 1]);
    assert_eq!(hints(&d, WAKE), 0, "no legacy per-seq hint files");
    assert!(!data_exists(&d, WAKE));
}
