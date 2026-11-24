//! Bounded pagination support shared by the regression cases.
use crate::cases::support::MemDisk;
use skrynia_native_service::cap::Disk;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct Calls {
    pub disk: MemDisk,
    pub calls: AtomicUsize,
}
impl Disk for Calls {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.disk.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.disk.write_atomic(p, b)
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.disk.remove(p)
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.disk.list(d)
    }
}

pub fn crafted(prefix: &[u8], tail: u16) -> [u8; 32] {
    let mut k = [0u8; 32];
    k[..prefix.len()].copy_from_slice(prefix);
    k[30..].copy_from_slice(&tail.to_be_bytes());
    k
}
