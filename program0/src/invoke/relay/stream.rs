//! READ / CLOSE on per-invocation descriptor handles (no bulk kernel import).
use super::{Ctx, INVALID, abi, alloc, complete, reply, word};

const EOF: i32 = -1;

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let fd = match fd as i32 {
        0 => c.input,
        1 => c.output,
        other => other,
    };
    if op == 4 {
        return if complete(f, *at) {
            reply(unsafe { abi::close(fd) }, &[])
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
    let mut out = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let b = unsafe { abi::read1(fd) };
        if b < 0 {
            return if b == EOF {
                reply(out.len() as i32, &out)
            } else {
                reply(b, &[])
            };
        }
        out.push(b as u8);
    }
    reply(out.len() as i32, &out)
}
