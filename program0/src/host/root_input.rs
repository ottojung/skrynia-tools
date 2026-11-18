#[cfg(target_arch = "wasm32")]
use super::root_abi as host;
#[cfg(target_arch = "wasm32")]
pub fn read_all(fd: i32) -> Result<Vec<u8>, i32> {
    let mut out = Vec::new();
    loop {
        let mut chunk = [0u8; 8192];
        let n = unsafe { host::read(fd, chunk.as_mut_ptr() as u32, chunk.len() as u32) };
        if n < 0 {
            return Err(n);
        }
        if n == 0 {
            return Ok(out);
        }
        if out
            .len()
            .checked_add(n as usize)
            .is_none_or(|x| x > 16 * 1024 * 1024)
        {
            return Err(-10);
        }
        out.extend_from_slice(&chunk[..n as usize]);
    }
}
