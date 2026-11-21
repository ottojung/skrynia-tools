#[cfg(target_arch = "wasm32")]
use super::{Eval, HostEval, entry, host};
#[cfg(target_arch = "wasm32")]
const RES_CAP: usize = 1024 * 1024;

// P0 must instantiate this service via skrynia.eval_delegate: the kernel links
// eval_buffered only into eval-delegated mediator contexts. The child P2 is an
// untrusted leaf; the kernel rejects any eval import in it (fail-closed).
#[cfg(target_arch = "wasm32")]
impl Eval for HostEval {
    type Err = i32;
    fn eval(
        &self,
        code: &[u8],
        input: &[u8],
        cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        let cb_idx = if cb.is_some() {
            entry::callback_index()
        } else {
            -1
        };
        let mut out = vec![0u8; RES_CAP];
        let n = unsafe {
            host::eval_buffered(
                code.as_ptr() as u32,
                code.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                out.as_mut_ptr() as u32,
                out.len() as u32,
                cb_idx,
            )
        };
        if n < 0 {
            return Err(n);
        }
        out.truncate(n as usize);
        Ok(out)
    }
}
