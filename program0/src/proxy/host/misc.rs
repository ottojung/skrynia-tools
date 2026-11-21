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
pub fn random(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(bits) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || bits == 0 || bits > 32768 {
        return wire::reply(wire::INVALID, &[]);
    }
    let n = ((bits as usize) + 7) / 8;
    let mut out = vec![0u8; n];
    let rc = unsafe { abi::secure_random_bits(out.as_mut_ptr() as u32, bits) };
    if rc < 0 {
        wire::reply(rc, &[])
    } else {
        wire::reply(n as i32, &out)
    }
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
