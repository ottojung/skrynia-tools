//! NEXT_ENTRY (op 9): kernel writes the name into caller-owned shared RAM;
//! P0 pumps those bytes back through the reply stream, never a linear pointer.
use super::{Ctx, INVALID, abi, alloc, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(fd) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let Some(cap) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let cap = (cap as u64).min(4096);
    let dst = c.pos;
    if dst.saturating_add(cap) > c.base + alloc::SLOT_SIZE {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let n = unsafe { abi::next_entry(fd as i32, dst, cap) };
    if n < 0 {
        return reply(n, &[]);
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
        out.push(b as u8);
    }
    c.pos = dst.saturating_add(n as u64);
    reply(n, &out)
}
