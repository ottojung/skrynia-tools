mod close;
mod open;
mod read;
use skrynia_v2_provider::{Callback, InvokeError};
use std::cell::RefCell;
#[derive(Default)]
pub struct Fake {
    state: RefCell<(bool, usize)>,
}
impl Callback for Fake {
    fn invoke(&self, request: &[u8]) -> Result<Vec<u8>, InvokeError> {
        let Some((&op, tail)) = request.split_first() else {
            return Err(InvokeError::Unavailable);
        };
        match op {
            1 => open::call(self, tail),
            2 => read::call(self, tail),
            4 => close::call(self, tail),
            _ => Err(InvokeError::Unavailable),
        }
    }
}
