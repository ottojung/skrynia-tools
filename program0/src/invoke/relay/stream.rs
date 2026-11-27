//! READ / CLOSE on per-invocation descriptor handles.
use super::{Ctx, INVALID, abi, alloc, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    if op == 10 {
        return match (complete(f, *at), c.resolve(fd as i32)) {
            (true, Ok(fd)) => reply(unsafe { abi::sync(fd) }, &[]),
            (true, Err(e)) => reply(e, &[]),
            _ => reply(INVALID, &[]),
        };
    }
    if op == 4 {
        return if complete(f, *at) {
            reply(c.close_fd(fd as i32), &[])
        } else {
            reply(INVALID, &[])
        };
    }
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    if n > 64 * 1024 {
        return reply(alloc::ERR_RANGE, &[]);
    }
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    let fd = match c.resolve(fd as i32) {
        Ok(fd) => fd,
        Err(e) => return reply(e, &[]),
    };
    let mut out = vec![0u8; n as usize];
    let mut done = 0usize;
    while done < out.len() {
        let part = &mut out[done..];
        let n = unsafe { abi::read(fd, part.as_mut_ptr() as u64, part.len() as u64) };
        if n < 0 {
            return reply(n as i32, &[]);
        }
        if n == 0 {
            break;
        }
        if n as usize > part.len() {
            return reply(-5, &[]);
        }
        done += n as usize;
    }
    reply(done as i32, &out[..done])
}
