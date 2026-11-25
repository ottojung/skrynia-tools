//! Capability stubs for legacy/capacity regressions.
use skrynia_native_service::cap::{Eval, Random};

#[allow(dead_code)]
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
