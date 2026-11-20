#[cfg(target_arch = "wasm32")]
use crate::cap::Random;
#[cfg(target_arch = "wasm32")]
use crate::guest_sys::{call, proto};

#[cfg(target_arch = "wasm32")]
pub struct CbRandom;

#[cfg(target_arch = "wasm32")]
impl Random for CbRandom {
    type Err = i32;
    fn fill(&self, out: &mut [u8]) -> Result<(), i32> {
        let mut req = vec![proto::RANDOM_FILL];
        proto::put_u32(&mut req, out.len() as u32);
        let res = call::check_ok(call::call(&req)?)?;
        let mut i = 0usize;
        let bytes = proto::get_bytes(&res, &mut i).ok_or(-1)?;
        if bytes.len() != out.len() {
            return Err(-1);
        }
        out.copy_from_slice(&bytes);
        Ok(())
    }
}
