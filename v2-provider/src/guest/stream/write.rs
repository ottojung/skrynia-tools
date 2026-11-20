use super::write;
pub fn send(fd: i32, mut bytes: &[u8]) -> Result<(), i32> {
    while !bytes.is_empty() {
        let n = unsafe { write(fd, bytes.as_ptr() as u32, bytes.len().min(16384) as u32) };
        if n <= 0 {
            return Err(if n == 0 { -5 } else { n });
        }
        bytes = &bytes[n as usize..];
    }
    Ok(())
}
