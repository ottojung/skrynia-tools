use super::{scope, wire};
use crate::proxy::abi;
pub fn open(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::OPEN_DIR) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let Some(real_path) = scope::map(path) else {
        return wire::reply(wire::DENIED, &[]);
    };
    let fd = unsafe { abi::open_directory(real_path.as_ptr() as u32, real_path.len() as u32) };
    if fd < 0 {
        return wire::reply(fd, &[]);
    }
    match scope::fd_new(fd) {
        Some(h) => wire::reply(h, &[]),
        None => {
            unsafe { abi::close(fd) };
            wire::reply(wire::DENIED, &[])
        }
    }
}
pub fn next(req: &[u8], cap: usize) -> Vec<u8> {
    if !scope::allowed(scope::op::NEXT) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let (Some(fd), Some(len)) = (wire::u32_at(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    if len > 4096 || len as usize > cap.saturating_sub(4) {
        return wire::reply(wire::OVERFLOW, &[]);
    }
    let Some(real) = scope::fd_get(fd as i32) else {
        return wire::reply(wire::DENIED, &[]);
    };
    let mut data = vec![0u8; len as usize];
    let n = unsafe { abi::next_entry(real, data.as_mut_ptr() as u32, len) };
    wire::reply(n, if n > 0 { &data[..n as usize] } else { &[] })
}
