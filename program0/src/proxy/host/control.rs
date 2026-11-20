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
pub fn random(req: &[u8], cap: usize) -> Vec<u8> {
    let mut at = 1;
    let Some(bits) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let len = (bits as u64).div_ceil(8) as usize;
    if len > wire::LIMIT || len > cap.saturating_sub(4) {
        return wire::reply(wire::OVERFLOW, &[]);
    }
    let mut data = vec![0u8; len];
    let code = unsafe { abi::secure_random_bits(data.as_mut_ptr() as u32, bits) };
    wire::reply(code, if code >= 0 { &data } else { &[] })
}
