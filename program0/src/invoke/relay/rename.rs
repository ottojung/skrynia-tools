//! RENAME (op 7): two staged paths, second in caller RAM after the code.
use super::{Ctx, INVALID, abi, alloc, code, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(n as usize);
    let Some(old) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    let a = c.base + alloc::SLOT_PATH_AT;
    if code::put_path(c.ram, &mut c.pos, a, old).is_err() {
        return reply(INVALID, &[]);
    }
    let Some(m) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(m as usize);
    let Some(new) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    let b = c.pos;
    if b.saturating_add(m as u64) > c.base + alloc::SLOT_SIZE
        || code::put(c.ram, &mut c.pos, b, new).is_err()
    {
        return reply(alloc::ERR_RANGE, &[]);
    }
    reply(unsafe { abi::rename(a, n as u64, b, m as u64) }, &[])
}
