use super::{Disk, MemDisk};
use crate::MutLock;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct RotateAtAcquire<'a> {
    disk: &'a MemDisk,
    path: String,
    next: Vec<u8>,
    fired: AtomicBool,
}
impl<'a> RotateAtAcquire<'a> {
    pub fn new(disk: &'a MemDisk, path: String, next: Vec<u8>) -> Self {
        Self {
            disk,
            path,
            next,
            fired: AtomicBool::new(false),
        }
    }
}
impl MutLock for RotateAtAcquire<'_> {
    type Err = ();
    fn try_acquire(&self) -> Result<bool, ()> {
        if !self.fired.swap(true, Ordering::SeqCst) {
            self.disk.write_atomic(&self.path, &self.next).unwrap();
        }
        Ok(true)
    }
    fn release(&self) -> Result<(), ()> {
        Ok(())
    }
}
