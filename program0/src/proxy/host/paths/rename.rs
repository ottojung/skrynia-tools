use super::{abi, wire};
pub fn rename(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let (Some(old), Some(new)) = (wire::bytes(req, &mut at), wire::bytes(req, &mut at)) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
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
