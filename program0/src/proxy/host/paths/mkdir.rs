use super::{abi, scope, wire};
pub fn mkdir(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::MKDIR) { return wire::reply(wire::DENIED, &[]); }
    let mut at = 1;
    let (Some(path), Some(mode)) = (wire::bytes(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() { return wire::reply(wire::INVALID, &[]); }
    let Some(rp) = scope::map(path) else { return wire::reply(wire::DENIED, &[]); };
    wire::reply(unsafe { abi::mkdir(rp.as_ptr() as u32, rp.len() as u32, mode) }, &[])
}
