//! Issue 5z scale, restart, ACK and least-privilege acceptance scenarios.
use super::issue5z::{A, B, put, wake_gens};
use super::{SENDER_A, WAKE, ack, ns, read_ok, seeded};
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::push;

/// One wake subscription and one sender cover both travellers and 226 threads.
pub fn both_keys() {
    let d = seeded();
    put(&d, A);
    put(&d, B);
    for i in 0..113 {
        put(&d, format!("thread-{i:03}-tes-v1").as_bytes());
        put(&d, format!("thread-{i:03}-vitalik-v1").as_bytes());
    }
    assert_eq!(wake_gens(&d), vec![228]);
    assert_eq!(super::hints(&d, WAKE), 0);
}

/// Durable generation survives restart; a stale ACK cannot rewind A-B-A.
pub fn restart_aba() {
    let d = seeded();
    put(&d, A);
    put(&d, B);
    assert_eq!(wake_gens(&d), vec![2]);
    ack(&d, &SENDER_A, 2);
    assert_eq!(wake_gens(&d), Vec::<u64>::new());
    put(&d, b"thread-0113-vitalik-v1");
    ack(&d, &SENDER_A, 1);
    assert_eq!(wake_gens(&d), vec![3]);
    assert!(
        d.read_all(&push::wake_slot_path("d", &ns(), WAKE))
            .unwrap()
            .is_some()
    );
}

/// Ambiguous/retried network responses: duplicate ACK is a no-op.
pub fn ambiguous() {
    let d = seeded();
    put(&d, A);
    ack(&d, &SENDER_A, 1);
    ack(&d, &SENDER_A, 1);
    assert!(!read_ok(&d, 1));
    put(&d, B);
    assert_eq!(wake_gens(&d), vec![2]);
}

/// A foreign namespace may write its own object but cannot wake this one.
pub fn foreign() {
    let d = seeded();
    let before = wake_gens(&d);
    assert!(super::issue5z::put_as(&d, &[9u8; 32], B));
    assert_eq!(wake_gens(&d), before);
    let path = push::wake_slot_path("d", &[9u8; 32], WAKE);
    assert!(d.read_all(&path).unwrap().is_none());
}
