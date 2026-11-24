use super::host::Host;
use super::path;
use super::{ERR_TICKET, ERR_VALIDATION};

const RETRIES: u8 = 8;
const EXISTS: i32 = -3;

fn transfer<H: Host>(
    host: &mut H,
    fd: i32,
    ticket: u32,
    publish: bool,
    rel: &[u8],
) -> Result<i32, i32> {
    host.write_all(fd, rel)?;
    host.finish(fd)?;
    Ok(host.invoke(publish, ticket))
}

/// One bounded-UTF-8 relative path, one fresh ticket, one named import.
/// The request file is removed on every exit path after creation.
pub fn call<H: Host>(host: &mut H, publish: bool, rel: &[u8]) -> i32 {
    if !path::is_public(rel) {
        return ERR_VALIDATION;
    }
    for _ in 0..RETRIES {
        let ticket = match host.random_u32() {
            Ok(t) => t,
            Err(e) => return e,
        };
        let fd = match host.create_request(ticket) {
            Ok(fd) => fd,
            Err(EXISTS) => continue,
            Err(e) => return e,
        };
        let outcome = transfer(host, fd, ticket, publish, rel);
        let cleaned = host.remove_request(ticket);
        return match (outcome, cleaned) {
            (Ok(code), Ok(())) => code,
            (Err(e), _) => e,
            (_, Err(e)) => e,
        };
    }
    ERR_TICKET
}
