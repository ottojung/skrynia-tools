#[cfg(target_arch = "wasm32")]
mod cb;
#[cfg(target_arch = "wasm32")]
mod entry;
#[cfg(target_arch = "wasm32")]
pub mod lock;
#[cfg(target_arch = "wasm32")]
use super::{cap::Eval, root_abi as host};
#[cfg(target_arch = "wasm32")]
pub use lock::HostLock;

#[cfg(target_arch = "wasm32")]
pub fn set_cb_invoke(invoke_id: Vec<u8>) {
    unsafe { cb::CB_INVOKE = Some(invoke_id) };
}
#[cfg(target_arch = "wasm32")]
pub fn clear_cb_invoke() {
    unsafe { cb::CB_INVOKE = None };
}

#[cfg(target_arch = "wasm32")]
pub struct HostEval;

#[cfg(target_arch = "wasm32")]
impl Eval for HostEval {
    type Err = i32;
    fn eval(
        &self,
        code: &[u8],
        input: &[u8],
        cb: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, i32> {
        let cb_idx = if cb.is_some() { 0 } else { -1 };
        let mut out = vec![0u8; 1024 * 1024];
        let n = unsafe {
            host::eval(
                code.as_ptr() as u32,
                code.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                cb_idx,
                out.as_mut_ptr() as u32,
                out.len() as u32,
            )
        };
        if n < 0 {
            return Err(n);
        }
        out.truncate(n as usize);
        Ok(out)
    }
}
