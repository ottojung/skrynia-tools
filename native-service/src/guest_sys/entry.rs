#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
use super::cap::{CbDisk, CbEval, CbLock, CbRandom};
#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
use crate::reg::Registry;

#[cfg(all(target_arch = "wasm32", feature = "cb-guest"))]
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(in_ptr: u32, in_len: u32, out_ptr: u32, out_cap: u32) -> i32 {
    let req = unsafe { core::slice::from_raw_parts(in_ptr as *const u8, in_len as usize) };
    let disk = CbDisk;
    let lock = CbLock;
    let rng = CbRandom;
    let eval = CbEval;
    let reg = match Registry::open(&disk, "programs") {
        Ok(r) => r,
        Err(_) => return -5,
    };
    let response = crate::handle::handle(&reg, &disk, &lock, &rng, &eval, req);
    let n = response.len();
    if n > out_cap as usize {
        return -10;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(response.as_ptr(), out_ptr as *mut u8, n);
    }
    n as i32
}
