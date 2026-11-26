//! OPEN_DISK / EXISTS: stage one framed path, relay the kernel reply verbatim.
use super::{Ctx, INVALID, abi, alloc, check_path, code, complete, reply, word};

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
    if code::put_path(c.ram, &mut c.pos, c.base, path).is_err() {
        return reply(-5, &[]);
    }
    let at_path = c.base + alloc::SLOT_PATH_AT;
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
