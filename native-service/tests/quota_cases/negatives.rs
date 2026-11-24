//! Cross-namespace scoped negatives stay fail-closed with no phantom wake.
use super::fixture::crossns;
use super::support::{KEY, WAKE_OWNER, list};
use crate::common::Mem;
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::{AppErr, WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;

type R = Result<[u8; 32], AppErr<(), ()>>;

fn put_w(d: &Mem, w: &[u8; 32], g: &[u8; 32], k: &[u8], wk: &[u8]) -> R {
    ops::put_wake(
        d,
        &NoLock,
        "d",
        WriteAuth::External(w),
        k,
        &AppPrecond::Absent,
        b"x",
        g,
        wk,
    )
}

#[test]
fn scoped_negatives_fail_closed_without_data_or_wake() {
    let (d, writer, grant) = crossns();
    let wk = push::WAKE_KEY;
    let before = list(&d, &WAKE_OWNER, wk);
    assert!(matches!(
        put_w(&d, &[3u8; 32], &grant, b"other-v1", wk),
        Err(AppErr::BadAuth)
    ));
    assert!(matches!(
        put_w(&d, &writer, &[99u8; 32], KEY, wk),
        Err(AppErr::BadAuth)
    ));
    assert!(matches!(
        put_w(&d, &writer, &grant, b"", wk),
        Err(AppErr::BadKey)
    ));
    assert!(matches!(
        put_w(&d, &writer, &grant, b"data-*", wk),
        Err(AppErr::BadKey)
    ));
    assert!(matches!(
        put_w(&d, &writer, &grant, KEY, b"prefix-key"),
        Err(AppErr::BadKey)
    ));
    assert_eq!(list(&d, &WAKE_OWNER, wk), before, "no phantom wake");
    assert!(super::support::no_data(&d, &writer, b"other-v1"));
}
