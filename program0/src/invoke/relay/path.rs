//! Framed descriptor ops 5/6/8/10 on existing fixed K0 imports.
//! No new kernel ABI and no application policy: bytes are staged in the
//! caller's own RAM slot and the kernel reply code is relayed verbatim.
use super::{Ctx, INVALID, abi, code, reply, word};

fn staged(c: &mut Ctx, f: &[u8], at: &mut usize) -> Option<(u64, u64)> {
    let n = word(f, at)? as u64;
    let end = at.saturating_add(n as usize);
    let bytes = f.get(*at..end)?;
    *at = end;
    let off = c.base + super::alloc::SLOT_PATH_AT;
    code::put_path(c.ram, &mut c.pos, off, bytes).ok()?;
    Some((off, n))
}

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize, op: u8) -> Vec<u8> {
    match op {
        5 | 6 | 8 => {
            let Some((off, n)) = staged(c, f, at) else {
                return reply(INVALID, &[]);
            };
            let mode = if op == 5 {
                word(f, at).unwrap_or(0) as i32
            } else {
                0
            };
            let rc = unsafe {
                match op {
                    5 => abi::mkdir(off, n, mode),
                    6 => abi::delete(off, n),
                    _ => abi::open_directory(off, n),
                }
            };
            reply(rc, &[])
        }
        10 => {
            let Some(fd) = word(f, at) else {
                return reply(INVALID, &[]);
            };
            reply(unsafe { abi::sync(fd as i32) }, &[])
        }
        _ => reply(super::UNSUPPORTED, &[]),
    }
}
