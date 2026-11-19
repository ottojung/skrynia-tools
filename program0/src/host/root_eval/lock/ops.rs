#[cfg(target_arch = "wasm32")]
mod w {
    use crate::root_abi as host;
    const LOCK_OFF: u64 = 0;
    static mut HELD: bool = false;
    fn cas(exp: u64, des: u64) -> Result<u64, i32> {
        let fd = unsafe { host::open_ram() };
        if fd < 0 {
            return Err(fd);
        }
        let mut cur: u64 = 0;
        let rc = unsafe { host::cas_ram_u64(fd, LOCK_OFF, exp, des, &mut cur as *mut u64 as u32) };
        let _ = unsafe { host::close(fd) };
        if rc != 0 { Err(rc) } else { Ok(cur) }
    }
    pub fn try_acquire() -> Result<bool, i32> {
        let cur = cas(0, 1)?;
        if cur == 0 {
            unsafe { HELD = true };
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn release() -> Result<(), i32> {
        if !unsafe { HELD } {
            return Err(-9);
        }
        let seen = cas(1, 0)?;
        unsafe { HELD = false };
        if seen == 1 { Ok(()) } else { Err(-9) }
    }
}
#[cfg(target_arch = "wasm32")]
pub use w::{release, try_acquire};
