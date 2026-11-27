use super::{Staging, abi, write_all};

pub fn write(fd: i32, bytes: &[u8]) -> Result<(), i32> {
    write_all(
        |ptr, len| unsafe { abi::write(fd, ptr as u64, len as u64) },
        bytes,
    )?;
    super::ops::sync(fd)
}

pub fn delete(stage: &mut Staging, path: &[u8]) -> Result<(), i32> {
    let (ptr, len) = stage.put(path)?;
    let rc = unsafe { abi::delete(ptr, len) };
    if rc == 0 { Ok(()) } else { Err(rc) }
}

pub fn invoke(publish: bool, ticket: u32) -> Option<i32> {
    // A trapping import has no response; the caller must report UNKNOWN.
    Some(unsafe {
        if publish {
            abi::publish(ticket as i32)
        } else {
            abi::unpublish(ticket as i32)
        }
    })
}
