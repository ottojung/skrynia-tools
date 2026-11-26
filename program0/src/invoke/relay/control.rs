//! Userland CAS locks and bounded no-callback nested eval relay.
use super::{Ctx, INVALID, UNSUPPORTED, abi, alloc, complete, nested, reply};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    match op {
        12 | 13 => {
            if !complete(f, *at) {
                return reply(INVALID, &[]);
            }
            let (want, want_set) = if op == 12 { (0, 1) } else { (1, 0) };
            let seen = unsafe { abi::cas1(c.ram, c.base + alloc::LOCK_AT, want, want_set) };
            reply(seen, &[])
        }
        15 => nested::call(c, f, at),
        _ => reply(UNSUPPORTED, &[]),
    }
}
