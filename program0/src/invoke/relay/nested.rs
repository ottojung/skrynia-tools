//! Bounded nested EVAL using private frame bytes and one supplied function.
use super::{Ctx, INVALID, abi, alloc, code, complete, index, reply, set, take, word};

fn eval(c: &mut Ctx, bytes: &[u8]) -> i64 {
    let child = Ctx {
        input: c.input,
        output: c.output,
        ram: c.ram,
        slot: c.slot,
        in_handle: c.in_handle,
        out_handle: c.out_handle,
        status: 1,
        nested: true,
        ..Ctx::default()
    };
    set(Some(child));
    let rc = unsafe {
        abi::eval(
            bytes.as_ptr() as u64,
            bytes.len() as u64,
            index(),
            c.in_handle,
            c.out_handle,
        )
    };
    // Nested handles and queues must never leak into the parent request.
    if let Some(mut child) = take() {
        child.close_all();
    }
    rc
}

pub fn call(c: &mut Ctx, f: &[u8], at: &mut usize) -> Vec<u8> {
    let Some(n) = word(f, at) else {
        return reply(INVALID, &[]);
    };
    let end = at.saturating_add(n as usize);
    let Some(bytes) = f.get(*at..end) else {
        return reply(INVALID, &[]);
    };
    *at = end;
    if !complete(f, *at) {
        return reply(INVALID, &[]);
    }
    if bytes.len() > code::MAX_CODE {
        return reply(alloc::ERR_RANGE, &[]);
    }
    reply(eval(c, bytes) as i32, &[])
}
