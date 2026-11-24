//! Fail-injecting Disk implementation.
use super::crash_support::CrashDisk;
use skrynia_native_service::cap::Disk;

impl Disk for CrashDisk {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.files.borrow().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        let (before, after) = self.mutation();
        if before {
            return Err(());
        }
        self.files.borrow_mut().insert(p.into(), b.into());
        if after { Err(()) } else { Ok(()) }
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        let (before, after) = self.mutation();
        if before {
            return Err(());
        }
        self.files.borrow_mut().remove(p);
        if after { Err(()) } else { Ok(()) }
    }
    fn list(&self, _: &str) -> Result<Vec<String>, ()> {
        panic!("unexpected enumeration")
    }
}
