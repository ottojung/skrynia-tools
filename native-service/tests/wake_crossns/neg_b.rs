use super::common::Mem;
use super::setup::{hints, wake_setup};
use skrynia_native_service::registry::appdata::push::PushErr;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, hash_bytes, read_cap};
#[test]
fn unauthorized_grant_issuance_rejected() {
    let d = Mem::default();
    let (ww, attacker, wg) = ([1u8; 32], [2u8; 32], [4u8; 32]);
    let wk = b"shared-wake-v1";
    push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Wake,
        &ww,
        &wg,
        wk,
        &read_cap(&attacker),
    )
    .unwrap();
    let e = push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Wake,
        &attacker,
        &wg,
        wk,
        &read_cap(&attacker),
    );
    assert!(
        matches!(e, Err(PushErr::BadAuth)),
        "cannot overwrite another ns grant"
    );
}
