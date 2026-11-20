use super::abi;
const MAX_INPUT: usize = 16 * 1024 * 1024;

pub fn receive(fd: i32) -> Result<Vec<u8>, i32> {
    let mut input = Vec::new();
    loop {
        let mut chunk = [0u8; 8192];
        let n = unsafe { abi::read(fd, chunk.as_mut_ptr() as u32, chunk.len() as u32) };
        if n < 0 {
            return Err(n);
        }
        if n == 0 {
            return Ok(input);
        }
        if input
            .len()
            .checked_add(n as usize)
            .is_none_or(|x| x > MAX_INPUT)
        {
            return Err(-10);
        }
        input.extend_from_slice(&chunk[..n as usize]);
    }
}
pub fn send(fd: i32, bytes: &[u8]) -> Result<(), i32> {
    let mut remaining = bytes;
    while !remaining.is_empty() {
        let n = unsafe { abi::write(fd, remaining.as_ptr() as u32, remaining.len() as u32) };
        if n <= 0 {
            return Err(if n == 0 { -5 } else { n });
        }
        remaining = &remaining[n as usize..];
    }
    Ok(())
}
