//! Generic userland resource lease and bounded no-callback nested eval relay.
use super::{Ctx, INVALID, UNSUPPORTED, alloc, complete, nested, reply};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    match op {
        // Ops 12/13 mediate one shared RAM byte, never a slot-local byte, so
        // every root request contends for the same P1-wide resource lease.
        // Reply: 1 acquired / 0 contended (12), 0 released / negative error
        // for a non-owner (13). Frame completeness is checked before any CAS.
        12 | 13 => {
            if !complete(f, *at) {
                return reply(INVALID, &[]);
            }
            if c.nested {
                return reply(UNSUPPORTED, &[]);
            }
            reply(alloc::mediated(c.ram, c.slot, op), &[])
        }
        15 => nested::call(c, f, at),
        _ => reply(UNSUPPORTED, &[]),
    }
}
