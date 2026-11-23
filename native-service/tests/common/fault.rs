//! Disk fault injection: deny removals, or deny only wake-slot slot writes.
use super::Mem;
use skrynia_native_service::cap::Disk;

pub struct NoRemove(pub Mem);
impl Disk for NoRemove {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.0.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.write_atomic(p, b)
    }
    fn remove(&self, _p: &str) -> Result<(), ()> {
        Err(()) // every remove is denied; the wake slot must never need one
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        self.0.list(d)
    }
}

pub struct NoSlotWrite(pub Mem);
impl Disk for NoSlotWrite {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.0.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        if p.contains("wake-slot.bin") {
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
