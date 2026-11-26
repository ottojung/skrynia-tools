//! Named path operations and byte-in/close descriptor relay.
use super::{Ctx, INVALID, abi, alloc, code, reply, word};

pub fn named(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(n as usize);
    let Some(path) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    if code::put_path(c.ram, &mut c.pos, c.base + alloc::SLOT_PATH_AT, path).is_err() {
        return reply(-5, &[]);
    }
    let at_path = c.base + alloc::SLOT_PATH_AT;
    // OPEN_DISK's trailing u32 mode is relayed verbatim; the kernel still
    // enforces root capabilities, so this adds no implicit authority.
    let mode = word(f, at).unwrap_or(1) as i32;
    let rc = unsafe {
        if op == 1 {
            abi::open_disk(at_path, n as u64, mode)
        } else {
            abi::exists(at_path, n as u64)
        }
    };
    reply(rc, &[])
}

pub fn stream(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let fd = match fd as i32 {
        0 => c.input,
        1 => c.output,
        other => other,
    };
    if op == 4 {
        return reply(unsafe { abi::close(fd) }, &[]);
    }
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let cap = (n as usize).min(64 * 1024);
    let mut out = Vec::with_capacity(cap);
    for _ in 0..cap {
        let b = unsafe { abi::read1(fd) };
        if b < 0 {
            break;
        }
        out.push(b as u8);
    }
    reply(out.len() as i32, &out)
}
