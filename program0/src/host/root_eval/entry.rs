#[cfg(target_arch = "wasm32")]
use super::cb;

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_guest_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    let iid = match unsafe { cb::CB_INVOKE.as_ref() } {
        Some(a) => a.clone(),
        None => return -13,
    };
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let res = match cb::dispatch(&iid, req) {
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
