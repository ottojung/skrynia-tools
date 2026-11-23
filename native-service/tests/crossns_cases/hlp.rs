use super::common::Mem;
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::{AppErr, WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;
type R = Result<[u8; 32], AppErr<(), ()>>;
pub fn pw(d: &Mem, w: &[u8; 32], k: &[u8], v: &[u8], g: &[u8; 32], wk: &[u8]) -> R {
    ops::put_wake(
        d,
        &NoLock,
        "d",
        WriteAuth::External(w),
        k,
        &AppPrecond::Absent,
        v,
        g,
        wk,
    )
}
pub fn pwf<D: skrynia_native_service::cap::Disk<Err = ()>>(
    d: &D,
    w: &[u8; 32],
    k: &[u8],
    g: &[u8; 32],
    wk: &[u8],
) -> bool {
    ops::put_wake(
        d,
        &NoLock,
        "d",
        WriteAuth::External(w),
        k,
        &AppPrecond::Absent,
        b"v",
        g,
        wk,
    )
    .is_err()
}
