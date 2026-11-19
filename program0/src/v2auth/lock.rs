#[cfg(target_arch = "wasm32")]
use super::host::root_abi as host;

#[cfg(target_arch = "wasm32")]
pub struct Guard(i32);

#[cfg(target_arch = "wasm32")]
pub fn acquire() -> Result<Guard, i32> {
    let fd = unsafe { host::open_ram() };
    if fd < 0 {
        return Err(fd);
    }
    if unsafe { host::truncate(fd, 8) } < 0 {
        unsafe { host::close(fd) };
        return Err(-5);
    }
    let mut obs = 0u64;
    for _ in 0..100000 {
        let rc = unsafe { host::cas_ram_u64(fd, 0, 0, 1, &mut obs as *mut u64 as u32) };
        if rc < 0 {
            unsafe { host::close(fd) };
            return Err(rc);
        }
        if obs == 0 {
            return Ok(Guard(fd));
        }
    }
    unsafe { host::close(fd) };
    Err(-10)
}

#[cfg(target_arch = "wasm32")]
impl Drop for Guard {
    fn drop(&mut self) {
        let mut obs = 0u64;
        unsafe { host::cas_ram_u64(self.0, 0, 1, 0, &mut obs as *mut u64 as u32) };
        unsafe { host::close(self.0) };
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Guard;
#[cfg(not(target_arch = "wasm32"))]
pub fn acquire() -> Result<Guard, i32> {
    Ok(Guard)
}
