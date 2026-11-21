use super::{abi, wire};
pub fn open(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe { abi::open_directory(path.as_ptr() as u32, path.len() as u32) },
        &[],
    )
}
pub fn next(req: &[u8], cap: usize) -> Vec<u8> {
    let mut at = 1;
    let (Some(fd), Some(len)) = (wire::u32_at(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || len as usize > wire::LIMIT || len as usize > cap.saturating_sub(4) {
        return wire::reply(wire::OVERFLOW, &[]);
    }
    let mut out = vec![0u8; len as usize];
    let n = unsafe { abi::next_entry(fd as i32, out.as_mut_ptr() as u32, len) };
    let b = if n >= 0 { &out[..n as usize] } else { &[] };
    wire::reply(n, b)
}
