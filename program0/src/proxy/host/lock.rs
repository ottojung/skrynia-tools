use super::{abi, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    if req.len() != 1 {
        return wire::reply(wire::INVALID, &[]);
    }
    let status = match req[0] {
        wire::MUTATION_TRY => unsafe { abi::try_mutation_lock() },
        wire::MUTATION_RELEASE => unsafe { abi::release_mutation_lock() },
        _ => wire::INVALID,
    };
    wire::reply(status, &[])
}
