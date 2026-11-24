use super::abi;
use super::host::{Abi, Host};
use super::{auth, bridge, frame};
const MAX_BODY: usize = 8192;

#[unsafe(no_mangle)]
pub extern "C" fn skrynia_main(input_fd: i32, output_fd: i32) -> i32 {
    let mut abi = Abi;
    let code = process(&mut abi, input_fd);
    let reply = frame::reply(code);
    let mut at = 0;
    while at < reply.len() {
        let n = unsafe {
            abi::write(
                output_fd,
                reply[at..].as_ptr() as u32,
                (reply.len() - at) as u32,
            )
        };
        if n <= 0 {
            return -5;
        }
        at += n as usize;
    }
    let _ = unsafe { abi::close(input_fd) };
    0
}

fn process<H: Host>(host: &mut H, input_fd: i32) -> i32 {
    let token = match auth::require(host, input_fd) {
        Ok(t) => t,
        Err(e) => return e,
    };
    let mut rest = Vec::new();
    loop {
        let mut chunk = [0u8; 4096];
        match host.read(input_fd, &mut chunk) {
            Ok(0) => break,
            Ok(n) if rest.len() + n <= MAX_BODY => rest.extend_from_slice(&chunk[..n]),
            Ok(_) => return bridge::INVALID,
            Err(e) => return e,
        }
    }
    let mut raw = Vec::with_capacity(33 + rest.len());
    raw.push(frame::VERSION);
    raw.extend_from_slice(&token);
    raw.extend_from_slice(&rest);
    let Some(request) = frame::decode(&raw) else {
        return bridge::INVALID;
    };
    bridge::run(host, &request)
}
