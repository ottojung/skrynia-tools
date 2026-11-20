use super::{abi, wire};
pub fn sync(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(fd) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { abi::sync(fd as i32) }, &[])
}
pub fn seek(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(fd), Some(off)) = (wire::u32_at(req, &mut at), wire::u64_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { abi::seek(fd as i32, off) }, &[])
}
pub fn truncate(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(fd), Some(sz)) = (wire::u32_at(req, &mut at), wire::u64_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { abi::truncate(fd as i32, sz) }, &[])
}
