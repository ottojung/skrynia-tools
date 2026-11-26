//! Framed descriptor ops 5/6/8/10 on existing fixed K0 imports.
//! No new kernel ABI and no application policy: bytes are staged in the
//! caller's own RAM slot and the kernel reply code is relayed verbatim.
use super::{Ctx, INVALID, UNSUPPORTED, abi, alloc, check_path, code, complete, reply, word};

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    match op {
        5 | 6 | 8 => {
            let Some(n) = word(f, at) else {
                return reply(INVALID, &[]);
            };
            let end = at.saturating_add(n as usize);
            let Some(path) = f.get(*at..end) else {
                return reply(INVALID, &[]);
            };
            *at = end;
            let mode = if op == 5 {
                let Some(m) = word(f, at) else {
                    return reply(INVALID, &[]);
                };
                m as i32
            } else {
                0
            };
            if !complete(f, *at) {
                return reply(INVALID, &[]);
            }
            if let Err(e) = check_path(path) {
                return reply(e, &[]);
            }
            let off = c.base + alloc::SLOT_PATH_AT;
            if code::put_path(c.ram, &mut c.pos, c.base, path).is_err() {
                return reply(-5, &[]);
            }
            let rc = unsafe {
                match op {
                    5 => abi::mkdir(off, n as u64, mode),
                    6 => abi::delete(off, n as u64),
                    _ => abi::open_directory(off, n as u64),
                }
            };
            if op == 8 {
                c.grant(rc);
            }
            reply(rc, &[])
        }
        10 => {
            let Some(fd) = word(f, at) else {
                return reply(INVALID, &[]);
            };
            if !complete(f, *at) {
                return reply(INVALID, &[]);
            }
            match c.resolve(fd as i32) {
                Ok(fd) => reply(unsafe { abi::sync(fd) }, &[]),
                Err(e) => reply(e, &[]),
            }
        }
        _ => reply(UNSUPPORTED, &[]),
    }
}
