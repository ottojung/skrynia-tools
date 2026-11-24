//! Decoupled notification regression: a full per-key hint queue must never
//! abort an authorized source CAS, and no wake is recorded for uncommitted data.
mod common;
mod quota_cases;

use common::issue5z::{B, put, wake_gens};
use common::{SENDER_A, WAKE, ns, seeded, subscribe};
use skrynia_native_service::NoLock;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::push;

#[test]
fn issue5z_1024_hint_pressure_never_blocks_writes() {
    let d = seeded();
    let key = b"pressure-key-v1";
    subscribe(&d, key);
    let ns = ns();
    for i in 0..1024 {
        d.write_atomic(&push::hint_path("d", &ns, key, i), &[7u8; 32])
            .unwrap();
    }
    assert!(d.list(&push::hints_root("d", &ns, key)).unwrap().len() >= 1024);
    put(&d, B);
    assert_eq!(
        wake_gens(&d),
        vec![1],
        "unrelated write is not blocked by pressure"
    );
}

#[test]
fn issue5z_foreign_unauthorised_keys_fail_closed() {
    common::issue5z_prims::foreign();
}

#[test]
fn issue5z_normal_sender_crosscheck() {
    let d = seeded();
    put(&d, B);
    let w = push::hint_list(&d, &NoLock, "d", &SENDER_A, &ns(), WAKE).unwrap();
    let seq = u64::from_le_bytes(w[4..12].try_into().unwrap());
    assert_eq!(seq, 1);
    assert!(push::hint_read(&d, &NoLock, "d", &SENDER_A, &ns(), WAKE, seq).is_ok());
    push::hint_ack(&d, &NoLock, "d", &SENDER_A, &ns(), WAKE, seq).unwrap();
    assert_eq!(wake_gens(&d), Vec::<u64>::new());
}
