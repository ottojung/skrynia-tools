use super::{abi, path, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(p) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    let Some(mode) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || mode & !0x1f != 0 || !path::valid(p) {
        return wire::reply(wire::DENIED, &[]);
    }
    let fd = unsafe { abi::open_disk(p.as_ptr() as u32, p.len() as u32, mode) };
    wire::reply(fd, &[])
}
