//! Bulk output and random-fill relay through private buffers.
mod random;
use super::{Ctx, INVALID, abi, alloc, code, complete, reply, word};

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
    let mut done = 0usize;
    while done < bytes.len() {
        let part = &bytes[done..];
        let n = unsafe { abi::write(fd, part.as_ptr() as u64, part.len() as u64) };
        if n < 0 {
            return reply(n as i32, &[]);
        }
        if n == 0 || n as usize > part.len() {
            return reply(-5, &[]);
        }
        done += n as usize;
    }
    reply(done as i32, &[])
}
