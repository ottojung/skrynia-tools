use super::{abi, scope, wire};
pub fn rename(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::RENAME) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let (Some(old), Some(new)) = (wire::bytes(req, &mut at), wire::bytes(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let (Some(ro), Some(rn)) = (scope::map(old), scope::map(new)) else {
        return wire::reply(wire::DENIED, &[]);
    };
    wire::reply(
        unsafe {
            abi::rename(
                ro.as_ptr() as u32,
                ro.len() as u32,
                rn.as_ptr() as u32,
                rn.len() as u32,
            )
        },
        &[],
    )
}
