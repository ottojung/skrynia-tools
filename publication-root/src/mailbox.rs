use super::host::Host;

// The pinned ABI 3.1 kernel maps openat EEXIST to E_IO (-5), the same scalar
// as genuine I/O failures. A failed exclusive create is a collision only when
// the ticket path actually exists; confirm before retrying, and never remove
// a ticket this invocation did not create.
const OPEN_E_IO: i32 = -5;

pub enum CreateError {
    Collision,
    Failed(i32),
}

pub fn write<H: Host>(host: &mut H, ticket: u32, path: &[u8]) -> Result<(), CreateError> {
    host.mkdir(super::path::mailbox_dir())
        .map_err(CreateError::Failed)?;
    let name = super::path::mailbox(ticket);
    // Create-only exclusive open: on Ok we created this file and own its
    // cleanup. A failed open is retried only if the path is really occupied.
    let fd = match host.open(&name, 15) {
        Ok(fd) => fd,
        Err(e) if e == OPEN_E_IO && host.exists(&name) => return Err(CreateError::Collision),
        Err(e) => return Err(CreateError::Failed(e)),
    };
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
