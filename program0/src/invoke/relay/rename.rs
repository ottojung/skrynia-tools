//! RENAME (op 7): two paths staged inside the caller's path area only.
use super::{Ctx, INVALID, abi, alloc, code, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(n as usize);
    let Some(old) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    let Some(m) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(m as usize);
    let Some(new) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    let a = c.base + alloc::SLOT_PATH_AT;
    // Stage both paths below the code area: a long second path must never
    // overwrite this slot's code or the next slot's CAS metadata.
    if code::put_path(c.ram, &mut c.pos, c.base, old).is_err() {
        return reply(alloc::ERR_RANGE, &[]);
    }
    let b = c.pos;
    let limit = c.base + alloc::SLOT_CODE_AT;
    if b > limit || code::put(c.ram, &mut c.pos, b, new, limit).is_err() {
        return reply(alloc::ERR_RANGE, &[]);
    }
    reply(unsafe { abi::rename(a, n as u64, b, m as u64) }, &[])
}
