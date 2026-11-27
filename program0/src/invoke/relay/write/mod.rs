//! Byte-out and random-fill relay (no bulk kernel import).
mod random;
use super::{Ctx, INVALID, abi, alloc, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    if op == 11 {
        return random::call(c, f, at);
    }
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    if n > 64 * 1024 {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let Some(bytes) = f.get(*at..at.saturating_add(n as usize)) else {
        return reply(INVALID, &[]);
    };
    *at += n as usize;
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    let fd = match c.resolve(fd as i32) {
        Ok(fd) => fd,
        Err(e) => return reply(e, &[]),
    };
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
