//! Negative gates: dynamic keys, reopen, revocation, no-sub, corruption.
use crate::common::*;
use skrynia_native_service::NoLock;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::push;

#[test]
fn fanout_covers_113_dynamic_thread_keys_with_one_subscription() {
    let d = seeded();
    for i in 0..113 {
        put(&d, format!("thread-{i:03}-tes-v1").as_bytes(), b"msg");
    }
    assert_eq!(list(&d), vec![113]);
    assert_eq!(hints(&d, b"thread-000-tes-v1"), 0, "no per-key subs");
}

#[test]
fn wake_slot_survives_reopen() {
    let d = seeded();
    put(&d, b"traveller-vitalik-v1", b"v");
    subscribe(&d, WAKE);
    assert_eq!(list(&d), vec![1], "wake state is durable");
}

#[test]
fn revoking_the_wake_subscription_stops_fanout() {
    let d = seeded();
    put(&d, b"traveller-tes-v1", b"v");
    let before = list(&d);
    unsubscribe(&d, WAKE);
    put(&d, b"traveller-vitalik-v1", b"v");
    assert_eq!(list(&d), before, "no advance after revocation");
}

#[test]
fn no_subscriber_means_no_slot_write() {
    let d = Mem::default();
    put(&d, b"unobserved-v1", b"v");
    assert!(
        d.read_all(&push::wake_slot_path("d", &ns(), WAKE))
            .unwrap()
            .is_none()
    );
}

#[test]
fn corrupt_slot_fails_closed() {
    let d = seeded();
    put(&d, b"data-v1", b"v");
    d.write_atomic(&push::wake_slot_path("d", &ns(), WAKE), b"garbage")
        .unwrap();
    assert!(matches!(
        push::hint_list(&d, &NoLock, "d", &SENDER_A, &ns(), WAKE),
        Err(push::PushErr::BadKey)
    ));
}
