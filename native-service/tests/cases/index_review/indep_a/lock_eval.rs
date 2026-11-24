//! Eval stub for lock regression.
use super::lock_support::Stub;
use skrynia_native_service::cap::Eval;

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
