//! Quota and remove-denial liveness of ordinary data writes.
use crate::common::*;
use skrynia_native_service::registry::appdata::push;

#[test]
fn wake_quota_never_blocks_ordinary_data_writes() {
    let d = seeded();
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
fn remove_denial_never_blocks_writes_or_hides_latest_wake() {
    let d = NoRemove(seeded());
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
