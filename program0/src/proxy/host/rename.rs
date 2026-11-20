use super::{abi, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(a), Some(b)) = (wire::bytes(req, &mut at), wire::bytes(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let n = unsafe {
        abi::rename(
            a.as_ptr() as u32,
            a.len() as u32,
            b.as_ptr() as u32,
            b.len() as u32,
        )
    };
    wire::reply(n, &[])
}
