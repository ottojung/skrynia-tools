//! P1 appdata write/subscription helpers shared by the wake regression tests.
use super::Mem;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, hash_bytes};

pub fn caps() -> ([u8; 32], [u8; 32], [u8; 32]) {
    ([3u8; 32], [7u8; 32], [11u8; 32])
}

pub fn subscribe(d: &Mem, key: &[u8]) {
    let (w, g, s) = caps();
    push::issue_grant::<Mem, NoLock>(
        d,
        &NoLock,
        "d",
        &push::Kind::Subscribe,
        &w,
        &g,
        key,
        &hash_bytes(&s),
    )
    .unwrap();
    push::subscribe(d, &NoLock, "d", &g, key, &s, b"endpoint").unwrap();
}

pub fn unsubscribe(d: &Mem, key: &[u8]) {
    let (_, g, s) = caps();
    push::unsubscribe(d, &NoLock, "d", &g, key, &s).unwrap();
}

pub fn put(d: &Mem, key: &[u8], v: &[u8]) {
    ops::put(
        d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        key,
        &AppPrecond::Absent,
        v,
    )
    .unwrap();
}
