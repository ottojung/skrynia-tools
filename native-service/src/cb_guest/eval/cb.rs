//! P1-owned delegated callback for nested guests.
//!
//! No function pointer or linear-memory buffer crosses stores: the callback
//! is reached only through CB_GET/CB_PUT byte frames relayed by the gateway.

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

/// Callback entry for an in-process userland interpreter (T6). The P0-mediated
/// path must not call it: P1 is suspended for the whole nested eval there.
#[allow(dead_code)]
pub fn handle(req: &[u8]) -> Result<Vec<u8>, i32> {
    let Some(id) = current_id() else {
        return Err(-13);
    };
    crate::registry::appdata::cb::guest_callback(
        &crate::cb_guest::cap::CbDisk,
        &crate::cb_guest::cap::CbLock,
        DIR,
        &id,
        req,
    )
}
