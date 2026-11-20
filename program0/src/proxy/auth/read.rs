use super::{DENIED, TOKEN_BYTES, VERIFIER_PATH, abi};

pub fn exact(fd: i32, into: &mut [u8]) -> Result<(), i32> {
    let mut offset = 0;
    while offset < into.len() {
        let n = unsafe {
            abi::read(
                fd,
                into[offset..].as_mut_ptr() as u32,
                (into.len() - offset) as u32,
            )
        };
        if n <= 0 {
            return Err(DENIED);
        }
        offset += n as usize;
    }
    Ok(())
}
pub fn verifier() -> Result<[u8; TOKEN_BYTES], i32> {
    let fd =
        unsafe { abi::open_disk(VERIFIER_PATH.as_ptr() as u32, VERIFIER_PATH.len() as u32, 1) };
    if fd < 0 {
        return Err(DENIED);
    }
    let mut expected = [0u8; TOKEN_BYTES];
    let ok = exact(fd, &mut expected).and_then(|()| {
        let mut extra = [0u8; 1];
        let n = unsafe { abi::read(fd, extra.as_mut_ptr() as u32, 1) };
        if n == 0 { Ok(()) } else { Err(DENIED) }
    });
    unsafe {
        abi::close(fd);
    }
    ok.map(|()| expected)
}
