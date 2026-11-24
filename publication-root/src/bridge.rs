use super::frame::Request;
use super::host::Host;
use super::{mailbox, path};

pub const OK: i32 = 0;
pub const INVALID: i32 = -1;
pub const MAILBOX: i32 = -2;
pub const UNKNOWN: i32 = -3;
pub const CLEANUP: i32 = -4;
const RETRIES: u8 = 8;
const EXISTS: i32 = -3;

/// One bounded path, one fresh ticket, one synchronous named import.
/// No queue and no effects after the reply is written.
pub fn run<H: Host>(host: &mut H, request: &Request) -> i32 {
    if !path::valid(&request.path) {
        return INVALID;
    }
    for _ in 0..RETRIES {
        let ticket = match host.random() {
            Ok(t) => t,
            Err(_) => return MAILBOX,
        };
        match mailbox::write(host, ticket, &request.path) {
            Err(EXISTS) => continue,
            Err(_) => return MAILBOX,
            Ok(()) => return finish(host, request.publish, ticket),
        }
    }
    MAILBOX
}

fn finish<H: Host>(host: &mut H, publish: bool, ticket: u32) -> i32 {
    let invoked = host.invoke(publish, ticket);
    let cleaned = mailbox::remove(host, ticket);
    match invoked {
        None => UNKNOWN,
        Some(code) if code == OK && cleaned.is_err() => CLEANUP,
        Some(code) => code,
    }
}
