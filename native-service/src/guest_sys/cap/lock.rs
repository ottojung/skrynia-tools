#[cfg(target_arch = "wasm32")]
use crate::MutLock;
#[cfg(target_arch = "wasm32")]
use crate::guest_sys::{call, proto};

#[cfg(target_arch = "wasm32")]
pub struct CbLock;

#[cfg(target_arch = "wasm32")]
impl MutLock for CbLock {
    type Err = i32;
    fn try_acquire(&self) -> Result<bool, i32> {
        let req = vec![proto::LOCK_ACQUIRE];
        let res = call::check_ok(call::call(&req)?)?;
        match res.first().copied() {
            Some(proto::TRUE) => Ok(true),
            Some(proto::FALSE) => Ok(false),
            _ => Err(-1),
        }
    }
    fn release(&self) -> Result<(), i32> {
        let req = vec![proto::LOCK_RELEASE];
        call::check_ok(call::call(&req)?).map(|_| ())
    }
}
