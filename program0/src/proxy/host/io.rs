use super::{scope, wire};
use crate::proxy::abi;
pub fn read(req: &[u8], capacity: usize) -> Vec<u8> {
    if !scope::allowed(scope::op::READ) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let (Some(fd), Some(len)) = (wire::u32_at(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    if len as usize > wire::LIMIT || len as usize > capacity.saturating_sub(4) {
        return wire::reply(wire::OVERFLOW, &[]);
    }
    let Some(real) = scope::fd_get(fd as i32) else {
        return wire::reply(wire::DENIED, &[]);
    };
    let mut out = vec![0u8; len as usize];
    let n = unsafe { abi::read(real, out.as_mut_ptr() as u32, len) };
    let bytes = if n >= 0 { &out[..n as usize] } else { &[] };
    wire::reply(n, bytes)
}
pub fn write(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::WRITE) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let Some(fd) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    let Some(bytes) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || bytes.len() > wire::LIMIT {
        return wire::reply(wire::INVALID, &[]);
    }
    let Some(real) = scope::fd_get(fd as i32) else {
        return wire::reply(wire::DENIED, &[]);
    };
    let n = unsafe { abi::write(real, bytes.as_ptr() as u32, bytes.len() as u32) };
    wire::reply(n, &[])
}
