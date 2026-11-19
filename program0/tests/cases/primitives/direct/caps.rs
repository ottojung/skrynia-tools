use super::{Disk, MemDisk};
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Default)]
pub struct NoList {
    disk: MemDisk,
    reads: AtomicUsize,
}
impl NoList {
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}
impl Disk for NoList {
    type Err = ();
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, ()> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.disk.read_all(path)
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), ()> {
        self.disk.write_atomic(path, bytes)
    }
    fn remove(&self, path: &str) -> Result<(), ()> {
        self.disk.remove(path)
    }
    fn list(&self, _: &str) -> Result<Vec<String>, ()> {
        panic!("root must never scan unrelated program records")
    }
}
