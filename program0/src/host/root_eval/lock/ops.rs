#[cfg(target_arch = "wasm32")]
use crate::root_abi as host;

#[cfg(target_arch = "wasm32")]
const LOCK_OFF: u64 = 0;

#[cfg(target_arch = "wasm32")]
pub fn try_acquire() -> Result<bool, i32> {
    // attempt to claim lock at shared RAM offset 0 using CAS (expect 0 -> 1)
    let mut cur: u64 = 0;
    let rc = unsafe { host::cas_ram_u64(LOCK_OFF, 0, 1, &mut cur as *mut u64) };
    if rc != 0 {
        return Err(rc);
    }
    Ok(cur == 0)
}

#[cfg(target_arch = "wasm32")]
pub fn release() -> Result<(), i32> {
    // best-effort release: if we hold it (value 1) set to 0; if someone else clobbered, signal error
    let mut cur: u64 = 0;
    let rc = unsafe { host::cas_ram_u64(LOCK_OFF, 1, 0, &mut cur as *mut u64) };
    if rc != 0 {
        return Err(rc);
    }
    if cur != 1 {
        // unexpected state - treat as error to fail closed
        return Err(-9);
    }
    Ok(())
}
