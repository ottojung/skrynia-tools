//! The single userland function P0 supplies: `p0.invoke(ctx,op,value)->i64`.
mod queue;
mod relay;
use super::{abi, alloc, code};
pub use queue::{Ctx, enter, feed_cb, leave, set, swap, take, with};

pub const GET: i32 = 0;
pub const PUT: i32 = 1;
pub const FLUSH: i32 = 2;
pub const CB_GET: i32 = 3;
pub const CB_PUT: i32 = 4;
pub const STATUS: i32 = 5;

fn step(c: &mut Ctx, op: i32, value: i32) -> i64 {
    match op {
        GET => c
            .reply
            .pop_front()
            .map(i64::from)
            .unwrap_or_else(|| unsafe { abi::read1(c.input) as i64 }),
        PUT if (0..=255).contains(&value) => {
            c.frame.push(value as u8);
            0
        }
        CB_GET => c.cb_in.pop_front().map(i64::from).unwrap_or(-1),
        CB_PUT if (0..=255).contains(&value) => {
            c.cb_out.push_back(value as u8);
            0
        }
        STATUS => i64::from(c.status),
        _ => -3,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn invoke(_ctx: i32, op: i32, value: i32) -> i64 {
    if op == FLUSH {
        let mut owned = take().unwrap_or_default();
        let rc = relay::run(&mut owned);
        set(Some(owned));
        return rc;
    }
    with(|c| step(c, op, value)).unwrap_or(-2)
}
#[used]
static INVOKE: extern "C" fn(i32, i32, i32) -> i64 = invoke;
pub fn index() -> i32 {
    invoke as *const () as usize as i32
}
