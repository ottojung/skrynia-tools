//! Shared test fakes and setup for T1-T5.
mod seed;

pub(super) use seed::seed_sender;

use super::Disk;
pub(super) use super::{MemDisk, caps, push};

pub(super) struct FailRemove<'a>(pub(super) &'a MemDisk);

impl Disk for FailRemove<'_> {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.0.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.write_atomic(p, b)
    }
    fn remove(&self, _p: &str) -> Result<(), ()> {
        Err(())
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        self.0.list(d)
    }
}
