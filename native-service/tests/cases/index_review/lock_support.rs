//! Lock-replay regression support.
use skrynia_native_service::{
    MutLock,
    cap::{Disk, Eval, Random},
    handle::handle,
    index,
    mlock::StdLock,
    reg::Registry,
    wdec::decode_res,
    wenc::encode_req,
    wire::{AdminRequest, Request},
};
use std::{cell::RefCell, collections::BTreeMap};

#[derive(Default)]
pub struct Mem(pub RefCell<BTreeMap<String, Vec<u8>>>);
impl Disk for Mem {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.borrow().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.borrow_mut().insert(p.into(), b.into());
        Ok(())
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.0.borrow_mut().remove(p);
        Ok(())
    }
    fn list(&self, _: &str) -> Result<Vec<String>, ()> {
        panic!("unexpected listing")
    }
}
pub struct Stub;
impl Random for Stub {
    type Err = ();
    fn fill(&self, b: &mut [u8]) -> Result<(), ()> {
        b.fill(9);
        Ok(())
    }
}
impl Eval for Stub {
    type Err = ();
    fn eval(
        &self,
        _: &[u8],
        _: &[u8],
        _: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        panic!("unexpected eval")
    }
}
