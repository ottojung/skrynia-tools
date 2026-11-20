use super::{abi, wire};
pub fn random(req: &[u8], capacity: usize) -> Vec<u8> {
    let mut at = 1;
    let Some(bits) = wire::u32_at(req, &mut at) else { return wire::reply(wire::INVALID, &[]); };
    if at != req.len() { return wire::reply(wire::INVALID, &[]); }
    let n = (bits as usize).div_ceil(8);
    if n > wire::LIMIT || n + 4 > capacity { return wire::reply(wire::OVERFLOW, &[]); }
    let mut out = vec![0u8; n];
    let rc = unsafe { abi::secure_random_bits(out.as_mut_ptr() as u32, bits) };
    if rc < 0 { wire::reply(rc, &[]) } else { wire::reply(n as i32, &out) }
}
pub fn open_ram(req: &[u8]) -> Vec<u8> {
    if req.len() != 1 { return wire::reply(wire::INVALID, &[]); }
    wire::reply(unsafe { abi::open_ram() }, &[])
}
pub fn cas_ram(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let p = (
        wire::u32_at(req, &mut at), wire::u64_at(req, &mut at),
        wire::u64_at(req, &mut at), wire::u64_at(req, &mut at),
    );
    let (Some(fd), Some(off), Some(exp), Some(des)) = p else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() { return wire::reply(wire::INVALID, &[]); }
    let mut old = [0u8; 8];
    let rc = unsafe { abi::cas_ram_u64(fd as i32, off, exp, des, old.as_mut_ptr() as u32) };
    if rc < 0 { wire::reply(rc, &[]) } else { wire::reply(0, &old) }
}
