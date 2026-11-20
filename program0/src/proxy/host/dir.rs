use super::{abi, wire};
pub fn open(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(p) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe { abi::open_directory(p.as_ptr() as u32, p.len() as u32) },
        &[],
    )
}
pub fn next(req: &[u8], capacity: usize) -> Vec<u8> {
    let mut at = 1;
    let (Some(fd), Some(cap)) = (wire::u32_at(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    if cap as usize > wire::LIMIT || cap as usize > capacity.saturating_sub(4) {
        return wire::reply(wire::OVERFLOW, &[]);
    }
    let mut out = vec![0u8; cap as usize];
    let n = unsafe { abi::next_entry(fd as i32, out.as_mut_ptr() as u32, cap) };
    let bytes = if n > 0 { &out[..n as usize] } else { &[] };
    wire::reply(n, bytes)
}
