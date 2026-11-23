//! Crash-before-commit gate: a failed value write leaves only a spurious wake.
mod common;
use common::*;
use skrynia_native_service::NoLock;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::{WriteAuth, ops};
use skrynia_native_service::wire::AppPrecond;

struct FailData(Mem);
impl Disk for FailData {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.0.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        if p.contains("/appdata/") {
            Err(())
        } else {
            self.0.write_atomic(p, b)
        }
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.0.remove(p)
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        self.0.list(d)
    }
}

#[test]
fn crash_before_value_commit_leaves_only_spurious_wake() {
    let mem = Mem::default();
    subscribe(&mem, WAKE);
    let d = FailData(mem);
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
    assert!(hints(&d.0, WAKE) >= 1, "wake hint durable before commit");
    assert!(
        !data_exists(&d.0, b"thread-001-vitalik-v1"),
        "no data without wake"
    );
}
