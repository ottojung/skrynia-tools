//! Authz gates: foreign writer, delete fan-out, exact sender grant.
use crate::common::*;
use skrynia_native_service::NoLock;
use skrynia_native_service::hash_bytes;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};

#[test]
fn foreign_writer_cannot_create_wake_state_in_another_namespace() {
    let d = seeded();
    ops::put(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&[9u8; 32]),
        b"thread-001-vitalik-v1",
        &skrynia_native_service::wire::AppPrecond::Absent,
        b"x",
    )
    .unwrap();
    assert!(list(&d).is_empty(), "foreign namespace must not wake ours");
}

#[test]
fn delete_also_advances_the_wake_generation() {
    let d = seeded();
    put(&d, b"traveller-tes-v1", b"v");
    let before = list(&d)[0];
    ops::delete(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        b"traveller-tes-v1",
        &hash_bytes(b"v"),
    )
    .unwrap();
    assert_eq!(list(&d), vec![before + 1]);
    assert!(read_ok(&d, before + 1));
}

#[test]
fn sender_without_exact_wake_grant_is_rejected() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    put(&d, b"data-v1", b"v");
    assert!(matches!(
        push::hint_list(&d, &NoLock, "d", &[99u8; 32], &ns(), WAKE),
        Err(push::PushErr::BadAuth)
    ));
}
