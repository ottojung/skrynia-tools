//! Wake regression suite for the fixed-slot reserved WAKE_KEY generation.
mod common;
mod wake_group;

use common::issue5z::{A, B, put, wake_gens};
use common::{FailData, WAKE, caps, data_exists, seeded, unsubscribe};
use skrynia_native_service::NoLock;
use skrynia_native_service::hash_bytes;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;

#[test]
fn issue5z_crash_after_staging_before_commit() {
    let d = FailData(seeded());
    let r = ops::put(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        B,
        &AppPrecond::Absent,
        b"v",
    );
    assert!(r.is_err(), "source commit must fail closed");
    assert_eq!(wake_gens(&d.0), vec![1]);
    assert!(!data_exists(&d.0, B));
}

/// Revocation stops fan-out; a rotated subscriber identity wakes again.
#[test]
fn issue5z_revocation_and_identity_rotation() {
    let d = seeded();
    put(&d, A);
    unsubscribe(&d, WAKE);
    let before = wake_gens(&d);
    put(&d, B);
    assert_eq!(wake_gens(&d), before);
    let (w, _, s) = caps();
    let rotated = [41u8; 32];
    push::issue_grant::<common::Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Subscribe,
        &w,
        &rotated,
        WAKE,
        &hash_bytes(&s),
    )
    .unwrap();
    push::subscribe(&d, &NoLock, "d", &rotated, WAKE, &s, b"rotated").unwrap();
    put(&d, b"thread-0113-vitalik-v1");
    assert_eq!(wake_gens(&d), vec![2]);
}
