//! Random request: [opcode 11][u32 bit count], reply [i32 byte count][bytes].
use super::{Ctx, INVALID, abi, alloc, code, complete, reply, word};

pub fn call(_c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(bits) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    let n = u64::from(bits).div_ceil(8);
    if n > code::MAX_CODE as u64 {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let mut out = vec![0u8; n as usize];
    let rc = unsafe { abi::secure_random_bits(out.as_mut_ptr() as u64, u64::from(bits)) };
    if rc != 0 {
        return reply(rc, &[]);
    }
    reply(n as i32, &out)
}
