use super::{abi, wire};
fn one(req: &[u8], f: unsafe extern "C" fn(u32, u32) -> i32) -> Vec<u8> {
    let mut at = 1;
    let Some(p) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(unsafe { f(p.as_ptr() as u32, p.len() as u32) }, &[])
}
pub fn delete(req: &[u8]) -> Vec<u8> {
    one(req, abi::delete)
}
pub fn exists(req: &[u8]) -> Vec<u8> {
    one(req, abi::exists)
}
pub fn is_dir(req: &[u8]) -> Vec<u8> {
    one(req, abi::is_dir)
}
pub fn mkdir(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(p), Some(m)) = (wire::bytes(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe { abi::mkdir(p.as_ptr() as u32, p.len() as u32, m) },
        &[],
    )
}
