use super::super::cap::{CbDisk, CbLock};
use crate::registry::appdata;

static mut INVOKE_ID: Option<Vec<u8>> = None;
const DIR: &str = "programs";

pub fn set_invoke(id: Vec<u8>) {
    unsafe {
        INVOKE_ID = Some(id);
    }
}

pub fn clear_invoke() {
    unsafe {
        INVOKE_ID = None;
    }
}

fn current_id() -> Option<Vec<u8>> {
    unsafe { (&*core::ptr::addr_of!(INVOKE_ID)).clone() }
}

/// Mediator callback for P2 -> P1 invoke requests under app-scoped delegated rights.
/// Called by kernel with provider (P1) memory staging: (req_ptr, req_len, res_ptr, res_cap).
#[unsafe(no_mangle)]
pub extern "C" fn skrynia_mediator_cb(rp: u32, rl: u32, dp: u32, dc: u32) -> i32 {
    let Some(iid) = current_id() else {
        return -13;
    };
    let req = unsafe { core::slice::from_raw_parts(rp as *const u8, rl as usize) };
    let res = match appdata::cb::guest_callback(&CbDisk, &CbLock, DIR, &iid, req) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if res.len() > dc as usize {
        return -11;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(res.as_ptr(), dp as *mut u8, res.len());
    }
    res.len() as i32
}

#[used]
static MEDIATOR_CB_PTR: extern "C" fn(u32, u32, u32, u32) -> i32 = skrynia_mediator_cb;

pub fn callback_index() -> i32 {
    skrynia_mediator_cb as *const () as usize as i32
}
