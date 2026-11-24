use super::abi;

pub fn remove(ticket: u32) -> Result<(), i32> {
    let p = super::path::ticket(ticket);
    let rc = unsafe { abi::delete(p.as_ptr() as u32, p.len() as u32) };
    match rc {
        0 | -2 => Ok(()),
        e if e < 0 => Err(e),
        _ => Err(super::ERR_CLEANUP),
    }
}

pub fn invoke(publish: bool, ticket: u32) -> i32 {
    unsafe {
        if publish {
            abi::publish(ticket)
        } else {
            abi::unpublish(ticket)
        }
    }
}
