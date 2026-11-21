use super::{abi, scope, wire};
fn write_ok(path: &[u8]) -> bool {
    scope::write_in(scope::PREFIX, scope::WRITE_OK, path)
}
pub fn mkdir(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(path), Some(mode)) = (wire::bytes(req, &mut at), wire::u32_at(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !write_ok(path) {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe { abi::mkdir(path.as_ptr() as u32, path.len() as u32, mode) },
        &[],
    )
}
pub fn delete(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !write_ok(path) {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe { abi::delete(path.as_ptr() as u32, path.len() as u32) },
        &[],
    )
}
pub fn rename(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(old), Some(new)) = (wire::bytes(req, &mut at), wire::bytes(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !write_ok(old) || !write_ok(new) {
        return wire::reply(wire::INVALID, &[]);
    }
    wire::reply(
        unsafe {
            abi::rename(
                old.as_ptr() as u32,
                old.len() as u32,
                new.as_ptr() as u32,
                new.len() as u32,
            )
        },
        &[],
    )
}
