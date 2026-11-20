use super::read;
pub fn receive(fd: i32) -> Result<Vec<u8>, i32> {
    let mut result = Vec::new();
    loop {
        let mut chunk = [0u8; 16384];
        let n = unsafe { read(fd, chunk.as_mut_ptr() as u32, chunk.len() as u32) };
        if n < 0 {
            return Err(n);
        }
        if n == 0 {
            return Ok(result);
        }
        if result.len() + n as usize > 8 * 1024 * 1024 {
            return Err(-10);
        }
        result.extend_from_slice(&chunk[..n as usize]);
    }
}
