//! Crash gates: failed data commit leaves spurious wake; failed slot aborts.
use crate::common::*;
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;

#[test]
fn crash_before_value_commit_leaves_only_spurious_wake() {
    let d = FailData(seeded());
    let res = ops::put(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        b"thread-001-vitalik-v1",
        &AppPrecond::Absent,
        b"x",
    );
    assert!(res.is_err(), "value write must fail closed");
    assert_eq!(list(&d.0), vec![1], "dirty slot durable before commit");
    assert!(!data_exists(&d.0, b"thread-001-vitalik-v1"));
}

#[test]
fn slot_write_failure_aborts_the_source_commit() {
    let mem = Mem::default();
    subscribe(&mem, WAKE);
    let d = NoSlotWrite(mem);
    let res = ops::put(
        &d,
        &NoLock,
        "d",
        WriteAuth::External(&caps().0),
        b"no-slot-v1",
        &AppPrecond::Absent,
        b"x",
    );
    assert!(res.is_err(), "source must not commit without durable slot");
    assert!(!data_exists(&d.0, b"no-slot-v1"));
}
