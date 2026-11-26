//! Framed generic descriptor relay. The i32-LE wire is userland, not kernel ABI.
mod control;
mod dir;
mod frame;
mod named;
mod nested;
mod path;
mod rename;
mod stream;
mod write;
use super::Ctx;
use super::{abi, alloc, check_path, code, index, set, take};
pub use frame::{complete, reply, word};

pub const INVALID: i32 = -1;
pub const UNSUPPORTED: i32 = -4;

fn dispatch(c: &mut Ctx, f: &[u8]) -> Vec<u8> {
    let Some(op) = f.first().copied() else {
        return reply(INVALID, &[]);
    };
    let mut at = 1usize;
    let out = match op {
        1 | 14 => named::call(c, f, &mut at, op),
        2 | 4 => stream::call(c, f, &mut at, op),
        3 | 11 => write::call(c, f, &mut at, op),
        5 | 6 | 8 => path::call(c, f, &mut at, op),
        10 => stream::call(c, f, &mut at, op),
        7 => rename::call(c, f, &mut at),
        9 => dir::call(c, f, &mut at),
        12 | 13 | 15 => control::call(c, f, &mut at, op),
        _ => return reply(UNSUPPORTED, &[]),
    };
    if complete(f, at) {
        out
    } else {
        reply(INVALID, &[])
    }
}
pub fn run(c: &mut Ctx) -> i64 {
    if c.reply_active {
        return -3;
    }
    if core::mem::take(&mut c.frame_failed) {
        return -3;
    }
    let frame = core::mem::take(&mut c.frame);
    let outcome = dispatch(c, &frame);
    c.status = 0;
    c.reply.extend(outcome);
    c.reply_active = true;
    0
}
