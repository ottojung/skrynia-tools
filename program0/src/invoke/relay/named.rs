//! OPEN_DISK / EXISTS: pass the private path buffer, relay the kernel reply.
use super::{Ctx, INVALID, abi, check_path, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(n as usize);
    let Some(path) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    let mode = if op == 1 {
        let Some(m) = word(f, at) else {
            return reply(INVALID, &[]);
        };
        m as i32
    } else {
        1
    };
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    if let Err(e) = check_path(path) {
        return reply(e, &[]);
    }
    let at_path = path.as_ptr() as u64;
    // OPEN_DISK's trailing u32 mode is relayed verbatim; the kernel still
    // enforces root capabilities, so this adds no implicit authority.
    let rc = unsafe {
        if op == 1 {
            abi::open_disk(at_path, n as u64, mode)
        } else {
            abi::exists(at_path, n as u64)
        }
    };
    if op == 1 {
        c.grant(rc);
    }
    reply(rc, &[])
}
