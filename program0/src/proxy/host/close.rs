use super::{abi, scope, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    let mut at = 1;
    let Some(fd) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() || !scope::allows(fd as i32) {
        return wire::reply(wire::INVALID, &[]);
    }
    // The kernel owns FD lifecycle/scope; keep the delegated allowlist
    // monotonic so a kernel-reused handle number stays authorized.
    wire::reply(unsafe { abi::close(fd as i32) }, &[])
}
