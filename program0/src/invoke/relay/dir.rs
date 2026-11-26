//! NEXT_ENTRY (op 9): kernel writes the name into caller-owned shared RAM;
//! P0 pumps those bytes back through the reply stream, never a linear pointer.
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
    let cap = cap as u64;
    let dst = c.base + alloc::SLOT_CODE_AT;
    if dst < c.base || dst.saturating_add(cap) > c.base + alloc::SLOT_SIZE {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let n = unsafe { abi::next_entry(fd, dst, cap) };
    if n < 0 {
        return reply(n, &[]);
    }
    if n as u64 > cap {
        return reply(alloc::ERR_RANGE, &[]);
    }
    if alloc::place(c.ram, &mut c.pos, dst).is_err() {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let mut out = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let b = unsafe { abi::read1(c.ram) };
        if b < 0 {
            return reply(b, &[]);
        }
        c.pos += 1;
        out.push(b as u8);
    }
    reply(n, &out)
}
