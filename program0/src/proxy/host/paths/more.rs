use super::{abi, path, wire};
pub fn rename(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(old), Some(newp)) = (wire::bytes(req, &mut at), wire::bytes(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !path::valid(old) || !path::valid(newp) {
        return wire::reply(wire::DENIED, &[]);
    }
    let n = unsafe {
        abi::rename(
            old.as_ptr() as u32,
            old.len() as u32,
            newp.as_ptr() as u32,
            newp.len() as u32,
        )
    };
    wire::reply(n, &[])
}
pub fn exists(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(p) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !path::valid(p) {
        return wire::reply(wire::DENIED, &[]);
    }
    wire::reply(
        unsafe { abi::exists(p.as_ptr() as u32, p.len() as u32) },
        &[],
    )
}
