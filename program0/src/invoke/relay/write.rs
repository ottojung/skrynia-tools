//! Byte-out and random-fill relay (no bulk kernel import).
use super::{Ctx, INVALID, abi, alloc, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let raw = fd as i32;
    let fd = match raw {
        0 => c.input,
        1 => c.output,
        _ => raw,
    };
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let cap = (n as usize).min(64 * 1024);
    if op == 11 {
        let dst = c.base + alloc::SLOT_CODE_AT;
        let rc = unsafe { abi::secure_random_bits(dst, n as u64) };
        if rc != 0 {
            return reply(rc, &[]);
        }
        return reply(n as i32, &[]);
    }
    let Some(bytes) = f.get(*at..at.saturating_add(cap)) else {
        return reply(INVALID, &[]);
    };
    *at += cap;
    let mut done = 0i32;
    for &b in bytes {
        let rc = unsafe { abi::write1(fd, i32::from(b)) };
        if rc != 0 {
            return reply(rc, &[]);
        }
        done += 1;
    }
    reply(done, &[])
}
