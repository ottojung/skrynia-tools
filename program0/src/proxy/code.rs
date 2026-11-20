use super::abi;
const GUEST: &[u8] = b"programs/root.wasm";
const MAX_CODE: usize = 8 * 1024 * 1024;

pub fn load() -> Result<Vec<u8>, i32> {
    read_from(GUEST)
}
pub fn read_from(path: &[u8]) -> Result<Vec<u8>, i32> {
    let fd = unsafe { abi::open_disk(path.as_ptr() as u32, path.len() as u32, 1) };
    if fd < 0 {
        return Err(fd);
    }
    let mut code = Vec::new();
    let result = loop {
        let mut chunk = [0u8; 8192];
        let n = unsafe { abi::read(fd, chunk.as_mut_ptr() as u32, chunk.len() as u32) };
        if n < 0 {
            break Err(n);
        }
        if n == 0 {
            break Ok(code);
        }
        if code
            .len()
            .checked_add(n as usize)
            .is_none_or(|x| x > MAX_CODE)
        {
            break Err(-10);
        }
        code.extend_from_slice(&chunk[..n as usize]);
    };
    unsafe {
        abi::close(fd);
    }
    result
}
