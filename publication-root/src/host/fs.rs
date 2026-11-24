use super::abi;

pub fn write(fd: i32, bytes: &[u8]) -> Result<(), i32> {
    let mut at = 0;
    while at < bytes.len() {
        let n = unsafe { abi::write(fd, bytes[at..].as_ptr() as u32, (bytes.len() - at) as u32) };
        if n <= 0 {
            return Err(if n == 0 { -5 } else { n });
        }
        at += n as usize;
    }
    let rc = unsafe { abi::sync(fd) };
    if rc < 0 { Err(rc) } else { Ok(()) }
}
pub fn delete(path: &[u8]) -> Result<(), i32> {
    let rc = unsafe { abi::delete(path.as_ptr() as u32, path.len() as u32) };
    if rc == 0 || rc == -2 { Ok(()) } else { Err(rc) }
}
pub fn invoke(publish: bool, ticket: u32) -> Option<i32> {
    // A trapping import has no response; the caller must report UNKNOWN.
    Some(unsafe {
        if publish {
            abi::publish(ticket)
        } else {
            abi::unpublish(ticket)
        }
    })
}
