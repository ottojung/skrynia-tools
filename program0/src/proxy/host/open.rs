use super::{abi, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    let Some(mode) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let fd = unsafe { abi::open_disk(path.as_ptr() as u32, path.len() as u32, mode) };
    wire::reply(fd, &[])
}
