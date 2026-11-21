#[cfg(target_arch = "wasm32")]
use super::root_abi as host;

#[cfg(target_arch = "wasm32")]
pub fn write_all(fd: i32, data: &[u8]) -> Result<(), i32> {
    let mut done = 0;
    while done < data.len() {
        let bytes = &data[done..];
        let n = unsafe { host::write(fd, bytes.as_ptr() as u32, bytes.len() as u32) };
        if n <= 0 {
            return Err(if n == 0 { -5 } else { n });
        }
        done += n as usize;
    }
    Ok(())
}
