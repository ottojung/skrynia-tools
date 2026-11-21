use super::{abi, scope, wire};
pub fn exists(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::EXISTS) { return wire::reply(wire::DENIED, &[]); }
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else { return wire::reply(wire::INVALID, &[]); };
    if at != req.len() { return wire::reply(wire::INVALID, &[]); }
    let Some(rp) = scope::map(path) else { return wire::reply(wire::DENIED, &[]); };
    wire::reply(unsafe { abi::exists(rp.as_ptr() as u32, rp.len() as u32) }, &[])
}
