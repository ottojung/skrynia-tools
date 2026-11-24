use super::host::Host;

pub fn write<H: Host>(host: &mut H, ticket: u32, path: &[u8]) -> Result<(), i32> {
    host.mkdir(super::path::mailbox_dir())?;
    let name = super::path::mailbox(ticket);
    // Create-only exclusive open: on Ok we created this file and own its
    // cleanup. A colliding ticket returns EXISTS and we never touch that file.
    let fd = host.open(&name, 15)?;
    let written = host.write(fd, path);
    let closed = host.close(fd);
    if let Err(err) = written {
        let _ = host.delete(&name);
        return Err(err);
    }
    if let Err(err) = closed {
        let _ = host.delete(&name);
        return Err(err);
    }
    Ok(())
}

pub fn remove<H: Host>(host: &mut H, ticket: u32) -> Result<(), i32> {
    host.delete(&super::path::mailbox(ticket))
}
