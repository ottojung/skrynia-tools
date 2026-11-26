//! Framed generic descriptor relay. The i32-LE wire is userland, not kernel ABI.
mod control;
mod dir;
mod io;
mod path;
mod rename;
mod write;
use super::Ctx;
use super::{abi, alloc, code, index, set, with};

pub const INVALID: i32 = -1;
pub const UNSUPPORTED: i32 = -4;

pub fn word(f: &[u8], at: &mut usize) -> Option<u32> {
    let raw: [u8; 4] = f.get(*at..*at + 4)?.try_into().ok()?;
    *at += 4;
    Some(u32::from_le_bytes(raw))
}
pub fn reply(code: i32, data: &[u8]) -> Vec<u8> {
    let mut out = code.to_le_bytes().to_vec();
    if code >= 0 {
        out.extend_from_slice(data)
    }
    out
}
fn dispatch(c: &mut Ctx, f: &[u8]) -> Vec<u8> {
    let Some(op) = f.first().copied() else {
        return reply(INVALID, &[]);
    };
    let mut at = 1usize;
    match op {
        1 | 14 => io::named(c, f, &mut at, op),
        2 | 4 => io::stream(c, f, &mut at, op),
        3 | 11 => write::call(c, f, &mut at, op),
        5 | 6 | 8 | 10 => path::call(c, f, &mut at, op),
        7 => rename::call(c, f, &mut at),
        9 => dir::call(c, f, &mut at),
        12 | 13 | 15 => control::call(c, f, &mut at, op),
        _ => reply(UNSUPPORTED, &[]),
    }
}
pub fn run(c: &mut Ctx) -> i64 {
    let frame = core::mem::take(&mut c.frame);
    let outcome = dispatch(c, &frame);
    c.status = 0;
    c.reply.extend(outcome);
    0
}
