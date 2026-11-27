//! Bounded no-callback nested EVAL: one `_start` child, one shared RAM slot.
use super::{Ctx, INVALID, abi, alloc, code, complete, index, reply, set, take, word};

fn eval(c: &mut Ctx, n: u32) -> i64 {
    let child = Ctx {
        pos: c.pos,
        input: c.input,
        output: c.output,
        ram: c.ram,
        base: c.base,
        in_handle: c.in_handle,
        out_handle: c.out_handle,
        status: 1,
        nested: true,
        ..Ctx::default()
    };
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
    // The child owns this same slot and fd cursor. Adopt its final cursor on
    // both success and trap, then drop its frame/reply/callback queues: they
    // must never outlive this evaluation or leak into the parent request.
    if let Some(mut child) = take() {
        c.pos = child.pos;
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
    if n as u64 > alloc::SLOT_SIZE - alloc::SLOT_CODE_AT {
        return reply(alloc::ERR_RANGE, &[]);
    }
    if code::put_code(c.ram, &mut c.pos, c.base, bytes).is_err() {
        return reply(alloc::ERR_RANGE, &[]);
    }
    reply(eval(c, n) as i32, &[])
}
