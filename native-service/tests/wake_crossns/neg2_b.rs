use super::common::Mem;
use super::hlp::pwf;
use super::setup::{hints, wake_setup};
use skrynia_native_service::cap::Disk;
use skrynia_native_service::read_cap;
use skrynia_native_service::registry::appdata::path;

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
fn failed_commit_spurious_only() {
    let (mem, wa, _, ga, _) = wake_setup();
    let wk = b"shared-wake-v1";
    let wn = read_cap(&[1u8; 32]);
    let d = FailData(mem);
    assert!(pwf(&d, &wa, b"k", &ga, wk));
    assert!(hints(&d.0, &wn, wk) >= 1);
    let dp = path::data_path("d", &read_cap(&wa), b"k").unwrap();
    assert!(d.0.read_all(&dp).unwrap().is_none());
}
