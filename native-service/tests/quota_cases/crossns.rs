//! Cross-namespace wake with a full source hint queue: commits stay available.
use super::fixture::crossns;
use super::support::{KEY, WAKE_OWNER, list};
use crate::common::Mem;
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::push::MAX_HINTS;
use skrynia_native_service::registry::appdata::{AppErr, WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;

type R = Result<[u8; 32], AppErr<(), ()>>;

fn put_w(d: &Mem, w: &[u8; 32], g: &[u8; 32], k: &[u8], e: &AppPrecond, v: &[u8]) -> R {
    ops::put_wake(
        d,
        &NoLock,
        "d",
        WriteAuth::External(w),
        k,
        e,
        v,
        g,
        push::WAKE_KEY,
    )
}

#[test]
fn crossns_full_source_queue_never_aborts_commits() {
    let (d, writer, grant) = crossns();
    let n = MAX_HINTS + 8;
    let mut prev: Option<[u8; 32]> = None;
    for i in 0..n {
        let v: &[u8] = if i % 2 == 0 { b"a" } else { b"b" };
        let e = match prev {
            Some(h) => AppPrecond::Present(h.to_vec()),
            None => AppPrecond::Absent,
        };
        prev =
            Some(put_w(&d, &writer, &grant, KEY, &e, v).unwrap_or_else(|x| panic!("{i}: {x:?}")));
    }
    assert_eq!(list(&d, &WAKE_OWNER, push::WAKE_KEY), vec![n as u64]);
}
