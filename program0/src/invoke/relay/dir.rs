//! NEXT_ENTRY (op 9): read the name directly into private memory.
use super::{Ctx, INVALID, abi, alloc, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let Some(cap) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    if cap > 4096 {
        return reply(alloc::ERR_RANGE, &[]);
    }
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    let fd = match c.resolve(fd as i32) {
        Ok(fd) => fd,
        Err(e) => return reply(e, &[]),
    };
    let mut out = vec![0u8; cap as usize];
    let n = unsafe { abi::next_entry(fd, out.as_mut_ptr() as u64, cap as u64) };
    if n < 0 {
        return reply(n, &[]);
    }
    if n as usize > out.len() {
        return reply(alloc::ERR_RANGE, &[]);
    }
    reply(n, &out[..n as usize])
}
