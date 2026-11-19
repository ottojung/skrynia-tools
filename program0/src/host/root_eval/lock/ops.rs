#[cfg(target_arch = "wasm32")]
use crate::root_abi as host;

#[cfg(target_arch = "wasm32")]
const LOCK_OFF: u64 = 0;

#[cfg(target_arch = "wasm32")]
fn cas(exp: u64, des: u64) -> Result<u64, i32> {
    let fd = unsafe { host::open_ram() };
    if fd < 0 {
        return Err(fd);
    }
    let rc = unsafe { host::truncate(fd, LOCK_OFF + 8) };
    if rc != 0 {
        let _ = unsafe { host::close(fd) };
        return Err(rc);
    }
    let mut cur: u64 = 0;
    let rc = unsafe { host::cas_ram_u64(fd, LOCK_OFF, exp, des, &mut cur as *mut u64 as u32) };
    let _ = unsafe { host::close(fd) };
    if rc != 0 {
        return Err(rc);
    }
    Ok(cur)
}

#[cfg(target_arch = "wasm32")]
pub fn try_acquire() -> Result<bool, i32> {
    Ok(cas(0, 1)? == 0)
}

#[cfg(target_arch = "wasm32")]
pub fn release() -> Result<(), i32> {
    if cas(1, 0)? != 1 {
        return Err(-9);
    }
    Ok(())
}
