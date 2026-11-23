//! Disk fake whose data writes fail, to stage a crash before source commit.
use super::Mem;
use skrynia_native_service::cap::Disk;

pub struct FailData(pub Mem);
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
