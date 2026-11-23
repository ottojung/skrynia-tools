//! 1025+ offline writes: every source value commits with a full hint queue.
use super::support::KEY;
use crate::common::{Mem, caps, data_exists, hints, list, subscribe};
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::AppErr;
use skrynia_native_service::registry::appdata::push::MAX_HINTS;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;

type R = Result<[u8; 32], AppErr<(), ()>>;

fn put(d: &Mem, expect: &AppPrecond, value: &[u8]) -> R {
    ops::put(
        d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        KEY,
        expect,
        value,
    )
}

#[test]
fn full_data_hint_queue_never_aborts_source_commits() {
    let d = crate::common::seeded();
    subscribe(&d, KEY);
    let n = MAX_HINTS + 64;
    let mut prev: Option<[u8; 32]> = None;
    for i in 0..n {
        let value: &[u8] = if i % 2 == 0 { b"v0" } else { b"v1" };
        let expect = match prev {
            Some(h) => AppPrecond::Present(h.to_vec()),
            None => AppPrecond::Absent,
        };
        prev = Some(put(&d, &expect, value).unwrap_or_else(|e| panic!("write {i}: {e:?}")));
    }
    assert!(data_exists(&d, KEY), "last source value must be committed");
    assert_eq!(
        hints(&d, KEY),
        256,
        "per-key legacy queue stays fuel-bounded"
    );
    assert_eq!(list(&d), vec![n as u64], "truthful pending wake");
}
