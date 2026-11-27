//! Framed descriptor ops 5/6/8/10 on existing fixed K0 imports.
//! Paths already reside in the private framed request buffer.
use super::{Ctx, INVALID, UNSUPPORTED, abi, check_path, complete, reply, word};

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
            let off = path.as_ptr() as u64;
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
        _ => reply(UNSUPPORTED, &[]),
    }
}
