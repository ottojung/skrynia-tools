use super::host::Host;

pub fn write<H: Host>(host: &mut H, ticket: u32, path: &[u8]) -> Result<(), i32> {
    host.mkdir(super::path::mailbox_dir())?;
    let name = super::path::mailbox(ticket);
    let fd = host.create(&name)?;
    let written = host.write(fd, path).and_then(|()| host.sync(fd));
    let closed = host.close(fd);
    written?;
    closed
}

pub fn remove<H: Host>(host: &mut H, ticket: u32) -> Result<(), i32> {
    host.delete(&super::path::mailbox(ticket))
}
