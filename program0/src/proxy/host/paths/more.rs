use super::{abi, wire};
pub fn rename(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(old) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    let Some(newp) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
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
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe { abi::exists(path.as_ptr() as u32, path.len() as u32) },
        &[],
    )
}
