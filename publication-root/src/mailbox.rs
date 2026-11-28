use super::host::Host;

// The pinned ABI 3.1 kernel maps openat's EEXIST to E_IO (-5). Only an
// exclusive-create open may produce that collision code; write/close failures
// carry the same scalar value but are not ticket collisions.
const EEXIST: i32 = -5;

pub enum CreateError {
    Collision,
    Failed(i32),
}

pub fn write<H: Host>(host: &mut H, ticket: u32, path: &[u8]) -> Result<(), CreateError> {
    host.mkdir(super::path::mailbox_dir())
        .map_err(CreateError::Failed)?;
    let name = super::path::mailbox(ticket);
    // Create-only exclusive open: on Ok we created this file and own its
    // cleanup. A colliding ticket returns EEXIST and we never touch that file.
    let fd = host.open(&name, 15).map_err(|e| {
        if e == EEXIST {
            CreateError::Collision
        } else {
            CreateError::Failed(e)
        }
    })?;
    let written = host.write(fd, path);
    let closed = host.close(fd);
    if let Err(err) = written {
        let _ = host.delete(&name);
        return Err(CreateError::Failed(err));
    }
    if let Err(err) = closed {
        let _ = host.delete(&name);
        return Err(CreateError::Failed(err));
    }
    Ok(())
}

pub fn remove<H: Host>(host: &mut H, ticket: u32) -> Result<(), i32> {
    host.delete(&super::path::mailbox(ticket))
}
