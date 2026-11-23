//! Shared P1 appdata helpers for the wake fan-out integration tests.
#![allow(dead_code)]
mod memdisk;
mod queries;
mod wake;
pub use memdisk::Mem;
#[allow(unused_imports)]
pub use queries::{data_exists, hints};
#[allow(unused_imports)]
pub use wake::{SENDER_A, SENDER_B, ack, ns, pending, put_ok, seed_sender};

use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, hash_bytes};

pub const WAKE: &[u8] = b"skrynia-wake-v1";
pub fn caps() -> ([u8; 32], [u8; 32], [u8; 32]) {
    ([3u8; 32], [7u8; 32], [11u8; 32])
}
pub fn subscribe(d: &Mem, key: &[u8]) {
    let (w, g, s) = caps();
    let subject = hash_bytes(&s);
    push::issue_grant::<Mem, NoLock>(
        d,
        &NoLock,
        "d",
        &push::Kind::Subscribe,
        &w,
        &g,
        key,
        &subject,
    )
    .unwrap();
    push::subscribe(d, &NoLock, "d", &g, key, &s, b"endpoint").unwrap();
}
pub fn unsubscribe(d: &Mem, key: &[u8]) {
    let (_, g, s) = caps();
    push::unsubscribe(d, &NoLock, "d", &g, key, &s).unwrap();
}
pub fn put(d: &Mem, key: &[u8], v: &[u8]) {
    let w = caps().0;
    ops::put(
        d,
        &NoLock,
        "d",
        WriteAuth::External(&w),
        key,
        &AppPrecond::Absent,
        v,
    )
    .unwrap();
}
