//! Bounded in-memory Disk for exact index regression cases.
use skrynia_native_service::{
    cap::Disk,
    index::{self, Scope},
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};
#[derive(Default)]
pub struct Memory {
    pub files: RefCell<BTreeMap<String, Vec<u8>>>,
    pub reads: Cell<usize>,
}
impl Disk for Memory {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        self.reads.set(self.reads.get() + 1);
        Ok(self.files.borrow().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.files.borrow_mut().insert(p.into(), b.into());
        Ok(())
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.files.borrow_mut().remove(p);
        Ok(())
    }
    fn list(&self, _: &str) -> Result<Vec<String>, ()> {
        panic!("unexpected directory enumeration")
    }
}
pub fn key(n: u32) -> [u8; 32] {
    let mut b = [0; 32];
    b[28..].copy_from_slice(&n.to_be_bytes());
    b
}
