//! RENAME (op 7): two independent private path buffers.
use super::{Ctx, INVALID, abi, check_path, complete, reply, word};

pub fn call(_c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
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
    if let Err(e) = check_path(old).and_then(|()| check_path(new)) {
        return reply(e, &[]);
    }
    reply(
        unsafe { abi::rename(old.as_ptr() as u64, n as u64, new.as_ptr() as u64, m as u64) },
        &[],
    )
}
