//! P0-mediated nested `_start` eval without P1 callbacks.
//!
//! A callback-dependent P2 cannot be run here: P1 is suspended for the
//! duration of P0's eval, so a CB queue could not reach P1's callback. An
//! ACTIVE status therefore fails closed with ENOTSUP; such guests require
//! the separate in-process userland interpreter (T6) calling the `Eval`
//! callback closure directly inside P1's own store.

use super::ops;
use super::relay::u32_at;
use super::stream::{self, Transport};

pub fn call<T: Transport>(t: &T, ctx: i32, req: &[u8], cap: usize) -> Result<Vec<u8>, i64> {
    stream::send(t, ctx, req)?;
    stream::flush(t, ctx)?;
    let status = t.invoke(ctx, ops::STATUS, 0);
    if status != ops::IDLE {
        return Err(if status == ops::ACTIVE {
            ops::ERR_NOTSUP
        } else {
            status
        });
    }
    let raw = stream::recv(t, ctx, cap)?;
    let code = u32_at(&raw, 0)? as i32;
    if code < 0 {
        return Err(code as i64);
    }
    Ok(raw[4..].to_vec())
}
