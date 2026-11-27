use super::{Staging, abi, read_all};

pub fn open(stage: &mut Staging, path: &[u8], mode: u32) -> Result<i32, i32> {
    let (ptr, len) = stage.put(path)?;
    let fd = unsafe { abi::open_disk(ptr, len, mode) };
    if fd < 0 { Err(fd) } else { Ok(fd) }
}

pub fn read(fd: i32, dst: &mut [u8]) -> Result<usize, i32> {
    read_all(
        |ptr, len| unsafe { abi::read(fd, ptr as u64, len as u64) },
        dst,
    )
}

pub fn close(fd: i32) -> Result<(), i32> {
    let rc = unsafe { abi::close(fd) };
    if rc < 0 { Err(rc) } else { Ok(()) }
}

pub fn sync(fd: i32) -> Result<(), i32> {
    let rc = unsafe { abi::sync(fd) };
    if rc < 0 { Err(rc) } else { Ok(()) }
}

pub fn mkdir(stage: &mut Staging, path: &[u8]) -> Result<(), i32> {
    let (ptr, len) = stage.put(path)?;
    let rc = unsafe { abi::mkdir(ptr, len, 4) };
    if rc == 0 { Ok(()) } else { Err(rc) }
}

pub fn random(stage: &mut Staging) -> Result<u32, i32> {
    let mut bytes = [0u8; 4];
    stage.random(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}
