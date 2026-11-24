use super::{abi, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(fd) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { abi::close(fd as i32) }, &[])
}
