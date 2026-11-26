//! Userland CAS locks and no-callback nested eval relay.
use super::{Ctx, INVALID, UNSUPPORTED, abi, alloc, code, index, reply, set, with, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    match op {
        12 | 13 => {
            let (want, want_set) = if op == 12 { (0, 1) } else { (1, 0) };
            let seen = unsafe { abi::cas1(c.ram, c.base + alloc::LOCK_AT, want, want_set) };
            reply(seen, &[])
        }
        15 => {
            let Some(n) = word(f, at) else {
                return reply(INVALID, &[]);
            };
            let end = at.saturating_add(n as usize);
            let Some(bytes) = f.get(*at..end) else {
                return reply(INVALID, &[]);
            };
            *at = end;
            if code::put(c.ram, &mut c.pos, c.base + alloc::SLOT_CODE_AT, bytes).is_err() {
                return reply(-5, &[]);
            }
            let mut child = Ctx::default();
            child.pos = c.pos;
            child.input = c.input;
            child.output = c.output;
            child.ram = c.ram;
            child.base = c.base;
            child.in_handle = c.in_handle;
            child.out_handle = c.out_handle;
            child.status = 1;
            set(Some(child));
            let rc = unsafe {
                abi::eval(
                    c.base + alloc::SLOT_CODE_AT,
                    n as u64,
                    index(),
                    c.in_handle,
                    c.out_handle,
                )
            };
            // The nested guest shares the RAM fd: adopt its final cursor so
            // the parent's next relative seek lands at the intended offset.
            c.pos = with(|x| x.pos).unwrap_or(c.pos);
            set(None);
            c.status = if rc < 0 { 2 } else { 0 };
            reply(rc as i32, &[])
        }
        _ => reply(UNSUPPORTED, &[]),
    }
}
