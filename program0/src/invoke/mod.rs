//! The single userland function P0 supplies: `p0.invoke(ctx,op,value)->i64`.
mod access;
mod framing;
mod queue;
mod relay;
use super::{abi, alloc, code};
use access::check_path;
pub use queue::{Ctx, enter, leave, set, take, with};

pub const GET: i32 = 0;
pub const PUT: i32 = 1;
pub const FLUSH: i32 = 2;
pub const CB_GET: i32 = 3;
pub const CB_PUT: i32 = 4;
pub const STATUS: i32 = 5;

fn step(c: &mut Ctx, op: i32, value: i32) -> i64 {
    match op {
        GET if c.reply_active => match c.reply.pop_front() {
            Some(b) => i64::from(b),
            None => {
                c.reply_active = false;
                -1
            }
        },
        GET => unsafe { abi::read1(c.input) as i64 },
        PUT => c.put(value),
        CB_GET | CB_PUT => -4,
        STATUS => i64::from(c.status),
        _ => -3,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn invoke(_ctx: i32, op: i32, value: i32) -> i64 {
    if op == FLUSH {
        let Some(mut owned) = take() else {
            return -2;
        };
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
