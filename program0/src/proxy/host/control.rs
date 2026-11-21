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
pub fn try_lock(req: &[u8]) -> Vec<u8> {
    if req.len() != 1 {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { abi::try_mutation_lock() }, &[])
}
pub fn release_lock(req: &[u8]) -> Vec<u8> {
    if req.len() != 1 {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { abi::release_mutation_lock() }, &[])
}
pub fn random(req: &[u8], capacity: usize) -> Vec<u8> {
    let mut at = 1;
    let Some(bits) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let max_bits = capacity
        .saturating_sub(4)
        .saturating_mul(8)
        .min(wire::LIMIT * 8) as u32;
    let bits = bits.min(max_bits);
    let bytes_len = ((bits as usize) + 7) / 8;
    let mut out = vec![0u8; bytes_len];
    let n = unsafe { abi::secure_random_bits(out.as_mut_ptr() as u32, bits) };
    let data = if n >= 0 { &out[..n as usize] } else { &[] };
    wire::reply(n, data)
}
