//! Restart/recovery: the pending wake is truthful and stale-ACK safe.
use super::support::KEY;
use crate::common::{Mem, SENDER_A, ack, caps, list, subscribe};
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
fn restart_reports_truthful_pending_wake_and_stale_ack_is_safe() {
    let d = crate::common::seeded();
    subscribe(&d, KEY);
    let n = MAX_HINTS + 16;
    let mut prev: Option<[u8; 32]> = None;
    for i in 0..n {
        let v: &[u8] = if i % 2 == 0 { b"x" } else { b"y" };
        let e = match prev {
            Some(h) => AppPrecond::Present(h.to_vec()),
            None => AppPrecond::Absent,
        };
        prev = Some(put(&d, &e, v).unwrap_or_else(|x| panic!("{i}: {x:?}")));
    }
    let g = list(&d);
    assert_eq!(g, vec![n as u64]);
    let open = Mem(std::sync::Mutex::new(d.0.lock().unwrap().clone()));
    assert_eq!(list(&open), g, "pending wake survives restart");
    ack(&open, &SENDER_A, n as u64 - 1);
    assert_eq!(list(&open), g, "stale ACK must be a no-op");
    ack(&open, &SENDER_A, n as u64);
    assert!(list(&open).is_empty(), "current ACK clears once");
}
