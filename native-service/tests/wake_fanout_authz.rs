//! Authz/del gates: foreign writer and delete fan-out.
mod common;
use common::*;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::{NoLock, hash_bytes};

#[test]
fn foreign_writer_cannot_create_wake_hints_in_another_namespace() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    let foreign = [9u8; 32];
    ops::put(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&foreign),
        b"thread-001-vitalik-v1",
        &skrynia_native_service::wire::AppPrecond::Absent,
        b"x",
    )
    .unwrap();
    assert_eq!(hints(&d, WAKE), 0, "foreign namespace must not wake ours");
}

#[test]
fn delete_also_fans_out_to_wake_key() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    put(&d, b"traveller-tes-v1", b"v");
    let n = hints(&d, WAKE);
    ops::delete(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        b"traveller-tes-v1",
        &hash_bytes(b"v"),
    )
    .unwrap();
    assert!(hints(&d, WAKE) > n, "delete must also invalidate wake key");
}
