use super::{abi, scope, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(fd) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !scope::allows(fd as i32) {
        return wire::reply(wire::INVALID, &[]);
    }
    let result = unsafe { abi::close(fd as i32) };
    scope::revoke(fd as i32);
    wire::reply(result, &[])
}
