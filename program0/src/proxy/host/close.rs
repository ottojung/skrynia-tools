use super::{abi, scope, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::CLOSE) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let Some(fd) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let Some(real) = scope::fd_take(fd as i32) else {
        return wire::reply(wire::DENIED, &[]);
    };
    wire::reply(unsafe { abi::close(real) }, &[])
}
