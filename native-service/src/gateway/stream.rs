//! Byte-stream transport over a supplied `invoke` function.
//!
//! A request is written one byte at a time with PUT, committed once with
//! FLUSH, then read back one byte at a time with GET. There is no replay:
//! an uncertain FLUSH result is final and the caller reports the error.

use super::ops;

pub trait Transport {
    fn invoke(&self, ctx: i32, op: i32, value: i32) -> i64;
}

pub fn send<T: Transport>(t: &T, ctx: i32, bytes: &[u8]) -> Result<(), i64> {
    for &byte in bytes {
        if t.invoke(ctx, ops::PUT, byte as i32) < 0 {
            return Err(ops::ERR_INVAL);
        }
    }
    Ok(())
}

pub fn send_cb<T: Transport>(t: &T, ctx: i32, bytes: &[u8]) -> Result<(), i64> {
    for &byte in bytes {
        if t.invoke(ctx, ops::CB_PUT, byte as i32) < 0 {
            return Err(ops::ERR_INVAL);
        }
    }
    Ok(())
}

pub fn recv<T: Transport>(t: &T, ctx: i32, cap: usize) -> Result<Vec<u8>, i64> {
    let mut out = Vec::new();
    loop {
        let value = t.invoke(ctx, ops::GET, 0);
        if value == ops::EOF {
            return Ok(out);
        }
        if value < 0 {
            return Err(value);
        }
        if out.len() >= cap {
            return Err(ops::ERR_LIMIT);
        }
        out.push(value as u8);
    }
}

pub fn flush<T: Transport>(t: &T, ctx: i32) -> Result<(), i64> {
    let code = t.invoke(ctx, ops::FLUSH, 0);
    if code < 0 { Err(code) } else { Ok(()) }
}

pub fn call<T: Transport>(t: &T, ctx: i32, req: &[u8], cap: usize) -> Result<Vec<u8>, i64> {
    send(t, ctx, req)?;
    flush(t, ctx)?;
    recv(t, ctx, cap)
}
