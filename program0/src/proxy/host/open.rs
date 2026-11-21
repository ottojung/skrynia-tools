use super::{abi, scope, wire};
pub fn call(req: &[u8]) -> Vec<u8> {
    if !scope::allowed(scope::op::OPEN_DISK) {
        return wire::reply(wire::DENIED, &[]);
    }
    let mut at = 1;
    let Some(path) = wire::bytes(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    let Some(mode) = wire::u32_at(req, &mut at) else {
        return wire::reply(wire::INVALID, &[]);
    };
    if at != req.len() {
        return wire::reply(wire::INVALID, &[]);
    }
    let Some(real_path) = scope::map(path) else {
        return wire::reply(wire::DENIED, &[]);
    };
    let fd = unsafe { abi::open_disk(real_path.as_ptr() as u32, real_path.len() as u32, mode) };
    if fd < 0 {
        return wire::reply(fd, &[]);
    }
    match scope::fd_new(fd) {
        Some(h) => wire::reply(h, &[]),
        None => {
            unsafe { abi::close(fd) };
            wire::reply(wire::DENIED, &[])
        }
    }
}
