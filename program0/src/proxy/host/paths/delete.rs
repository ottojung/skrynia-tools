use super::{abi, scope, wire};
pub fn delete(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::DELETE) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let Some(rp) = scope::map(path) else {
        return wire::reply(wire::DENIED, &[]);
    };
    wire::reply(
        unsafe { abi::delete(rp.as_ptr() as u32, rp.len() as u32) },
        &[],
    )
}
