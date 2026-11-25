//! Disk decorator counting source-level capability work per operation.
use crate::cases::support::MemDisk;
use skrynia_native_service::cap::Disk;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Metered {
    inner: MemDisk,
    reads: AtomicUsize,
    lists: AtomicUsize,
    writes: AtomicUsize,
}

impl Metered {
    pub fn new() -> Self {
        Self {
            inner: MemDisk::default(),
            reads: AtomicUsize::new(0),
            lists: AtomicUsize::new(0),
            writes: AtomicUsize::new(0),
        }
    }

    /// (read ops, directory listings, bytes written)
    pub fn ops(&self) -> (usize, usize, usize) {
        (
            self.reads.load(Ordering::SeqCst),
            self.lists.load(Ordering::SeqCst),
            self.writes.load(Ordering::SeqCst),
        )
    }
}

impl Disk for Metered {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.writes.fetch_add(b.len(), Ordering::SeqCst);
        self.inner.write_atomic(p, b)
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.inner.remove(p)
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        self.lists.fetch_add(1, Ordering::SeqCst);
        self.inner.list(d)
    }
}
