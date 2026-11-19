#[cfg(target_arch = "wasm32")]
mod cb;
#[cfg(target_arch = "wasm32")]
use super::{cap::Eval, root_abi as host};

#[cfg(target_arch = "wasm32")]
pub fn set_cb_app(app: Vec<u8>) {
    unsafe { cb::CB_APP = Some(app) };
}
#[cfg(target_arch = "wasm32")]
pub fn clear_cb_app() {
    unsafe { cb::CB_APP = None };
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

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_guest_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    let app = match unsafe { cb::CB_APP.as_ref() } {
        Some(a) => a.clone(),
        None => return -13,
    };
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let res = match cb::dispatch(&app, req) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if res.len() > dc as usize {
        return -11;
    }
    unsafe { core::ptr::copy_nonoverlapping(res.as_ptr(), dp as *mut u8, res.len()) };
    res.len() as i32
}

#[cfg(target_arch = "wasm32")]
#[used]
static GUEST_CB_PTR: unsafe extern "C" fn(u32, u32, u32, u32) -> i32 = skrynia_guest_cb;
