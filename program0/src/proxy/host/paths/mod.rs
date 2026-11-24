mod more;
pub(super) use super::{abi, path, wire};
pub use more::{exists, rename};
pub fn mkdir(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(p), Some(flags)) = (wire::bytes(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !path::valid(p) {
        return wire::reply(wire::DENIED, &[]);
    }
    wire::reply(
        unsafe { abi::mkdir(p.as_ptr() as u32, p.len() as u32, flags) },
        &[],
    )
}
pub fn delete(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(p) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !path::valid(p) {
        return wire::reply(wire::DENIED, &[]);
    }
    wire::reply(
        unsafe { abi::delete(p.as_ptr() as u32, p.len() as u32) },
        &[],
    )
}
