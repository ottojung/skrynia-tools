#[cfg(target_arch = "wasm32")]
mod w {
    use crate::root_abi as host;
    const LOCK_OFF: u64 = 0;
    const STALE: u64 = 30_000_000;
    static mut HELD: u64 = 0;
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
    fn tick() -> u64 {
        let t = unsafe { host::now() } as u64;
        if t == 0 { 1 } else { t }
    }
    pub fn try_acquire() -> Result<bool, i32> {
        let me = tick();
        let cur = cas(0, me)?;
        if cur == 0 {
            unsafe { HELD = me };
            return Ok(true);
        }
        if tick().saturating_sub(cur) < STALE {
            return Ok(false);
        }
        let seen = cas(cur, me)?;
        Ok(if seen == cur {
            unsafe { HELD = me };
            true
        } else {
            false
        })
    }
    pub fn release() -> Result<(), i32> {
        let me = unsafe { HELD };
        if me == 0 {
            return Err(-9);
        }
        let seen = cas(me, 0)?;
        unsafe { HELD = 0 };
        if seen == me { Ok(()) } else { Err(-9) }
    }
}
#[cfg(target_arch = "wasm32")]
pub use w::{release, try_acquire};
