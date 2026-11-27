//! Random request: [opcode 11][u32 bit count], reply [i32 byte count][bytes].
use super::{Ctx, INVALID, abi, alloc, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(bits) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    let n = u64::from(bits).div_ceil(8);
    if n > alloc::SLOT_SIZE - alloc::SLOT_CODE_AT {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let dst = c.base + alloc::SLOT_CODE_AT;
    let rc = unsafe { abi::secure_random_bits(dst, u64::from(bits)) };
    if rc != 0 {
        return reply(rc, &[]);
    }
    if let Err(e) = alloc::place(c.ram, &mut c.pos, dst) {
        return reply(e, &[]);
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
    reply(n as i32, &out)
}
