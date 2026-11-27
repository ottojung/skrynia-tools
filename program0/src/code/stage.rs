use super::{GUEST_PATH, MAX_CODE, abi};

pub fn load() -> Result<Vec<u8>, i32> {
    let src = unsafe { abi::open_disk(GUEST_PATH.as_ptr() as u64, GUEST_PATH.len() as u64, 1) };
    if src < 0 {
        return Err(src);
    }
    let result = read_code(src);
    unsafe { abi::close(src) };
    result
}

fn read_code(src: i32) -> Result<Vec<u8>, i32> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let cap = (MAX_CODE - bytes.len()).min(chunk.len());
        if cap == 0 {
            return match unsafe { abi::read1(src) } {
                -1 => Ok(bytes),
                b if b < 0 => Err(b),
                _ => Err(-6),
            };
        }
        let n = unsafe { abi::read(src, chunk.as_mut_ptr() as u64, cap as u64) };
        if n < 0 {
            return Err(n as i32);
        }
        if n == 0 {
            return Ok(bytes);
        }
        let part = chunk
            .get(..cap)
            .and_then(|b| b.get(..n as usize))
            .ok_or(-5)?;
        bytes.try_reserve(part.len()).map_err(|_| -5)?;
        bytes.extend_from_slice(part);
    }
}
