use super::abi;

pub fn open(path: &[u8], mode: u32) -> Result<i32, i32> {
    let fd = unsafe { abi::open_disk(path.as_ptr() as u32, path.len() as u32, mode) };
    if fd < 0 { Err(fd) } else { Ok(fd) }
}
pub fn read(fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
    let n = unsafe { abi::read(fd, dst.as_mut_ptr() as u32, dst.len() as u32) };
    if n < 0 { Err(n) } else { Ok(n as usize) }
}
pub fn close(fd: i32) -> Result<(), i32> {
    let rc = unsafe { abi::close(fd) };
    if rc < 0 { Err(rc) } else { Ok(()) }
}
pub fn mkdir(path: &[u8]) -> Result<(), i32> {
    let rc = unsafe { abi::mkdir(path.as_ptr() as u32, path.len() as u32, 4) };
    if rc == 0 || rc == -3 { Ok(()) } else { Err(rc) }
}
pub fn random() -> Result<u32, i32> {
    let mut bytes = [0u8; 4];
    let rc = unsafe { abi::secure_random_bits(bytes.as_mut_ptr() as u32, 32) };
    if rc < 0 {
        Err(rc)
    } else {
        Ok(u32::from_le_bytes(bytes))
    }
}
