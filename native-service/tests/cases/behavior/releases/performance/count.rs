use crate::cases::support::MemDisk;
use skrynia_native_service::cap::Disk;
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Default)]
pub struct Counted {
    disk: MemDisk,
    read_bytes: AtomicUsize,
    list_ops: AtomicUsize,
    write_bytes: AtomicUsize,
}
impl Counted {
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.read_bytes.load(Ordering::SeqCst),
            self.list_ops.load(Ordering::SeqCst),
            self.write_bytes.load(Ordering::SeqCst),
        )
    }
}
impl Disk for Counted {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        let r = self.disk.read_all(p)?;
        self.read_bytes
            .fetch_add(r.as_ref().map_or(0, Vec::len), Ordering::SeqCst);
        Ok(r)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.write_bytes.fetch_add(b.len(), Ordering::SeqCst);
        self.disk.write_atomic(p, b)
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.disk.remove(p)
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        self.list_ops.fetch_add(1, Ordering::SeqCst);
        self.disk.list(d)
    }
}
