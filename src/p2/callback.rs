//! Synchronous P1-owned callback roundtrip and scalar byte queues.
//!
//! `CB_GET` passes one opaque request byte to P1's `Eval` callback and stores
//! its reply bytes; `CB_PUT` drains that reply. The call happens on the
//! evaluator's own stack, so P1 is never suspended waiting on a P0 relay.

use super::error::{EINVAL, ENOTSUP, EOF, ERANGE};
use super::state::HostState;

impl HostState<'_, '_> {
    pub(crate) fn get(&mut self) -> i64 {
        match self.input.get(self.at) {
            Some(byte) => {
                self.at += 1;
                i64::from(*byte)
            }
            None => i64::from(EOF),
        }
    }

    pub(crate) fn put(&mut self, value: i32) -> i64 {
        let Ok(byte) = u8::try_from(value) else {
            return i64::from(EINVAL);
        };
        if self.output.len() >= self.cap {
            return i64::from(ERANGE);
        }
        self.output.push(byte);
        0
    }

    pub(crate) fn reply_pending(&self) -> bool {
        self.reply_at < self.reply.len()
    }

    pub(crate) fn invoke_callback(&mut self, request: u8) -> Result<(), i32> {
        let Some(cb) = self.cb.as_mut() else {
            return Err(ENOTSUP);
        };
        let reply = cb(&[request])?;
        self.reply = reply;
        self.reply_at = 0;
        Ok(())
    }

    pub(crate) fn reply_byte(&mut self) -> i64 {
        match self.reply.get(self.reply_at) {
            Some(byte) => {
                self.reply_at += 1;
                i64::from(*byte)
            }
            None => i64::from(EOF),
        }
    }
}
