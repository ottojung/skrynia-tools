#[cfg(target_arch = "wasm32")]
use crate::cap::Eval;
#[cfg(target_arch = "wasm32")]
use crate::guest_sys::{call, proto};

#[cfg(target_arch = "wasm32")]
pub struct CbEval;

#[cfg(target_arch = "wasm32")]
impl Eval for CbEval {
    type Err = i32;
    fn eval(
        &self,
        wasm: &[u8],
        input: &[u8],
        _cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        let iid = crate::take_invoke_ctx().unwrap_or_default();
        let mut req = vec![proto::EVAL_APP];
        proto::put_bytes(&mut req, wasm);
        proto::put_bytes(&mut req, input);
        proto::put_bytes(&mut req, &iid);
        let res = call::check_ok(call::call(&req)?)?;
        let mut i = 0usize;
        proto::get_bytes(&res, &mut i).ok_or(-1)
    }
}
