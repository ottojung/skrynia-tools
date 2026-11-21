use super::{abi, wire};
pub fn read(req: &[u8], capacity: usize) -> Vec<u8> {
    let mut at = 1;
    let (Some(fd), Some(len)) = (wire::u32_at(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let max_len = capacity.saturating_sub(4).min(wire::LIMIT);
    let len = (len as usize).min(max_len) as u32;
    let mut out = vec![0u8; len as usize];
    let n = unsafe { abi::read(fd as i32, out.as_mut_ptr() as u32, len) };
    let bytes = if n >= 0 { &out[..n as usize] } else { &[] };
    wire::reply(n, bytes)
}
pub fn write(req: &[u8]) -> Vec<u8> {
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
    let n = unsafe { abi::write(fd as i32, bytes.as_ptr() as u32, bytes.len() as u32) };
    wire::reply(n, &[])
}
