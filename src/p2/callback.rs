//! Complete opaque callback frames execute synchronously in P1.
use super::error::{EINVAL, EIO, ENOTSUP, EOF, ERANGE};
use super::state::HostState;

impl HostState<'_, '_> {
    fn reply_pending(&self) -> bool {
        self.reply_at < self.reply.len()
    }

    pub(crate) fn callback_put(&mut self, value: i32) -> i64 {
        if self.reply_pending() {
            return self.fail(EIO);
        }
        let Ok(byte) = u8::try_from(value) else {
            return self.fail(EINVAL);
        };
        if self.request.len() >= self.callback_cap {
            return self.fail(ERANGE);
        }
        self.request.push(byte);
        0
    }

    pub(crate) fn callback_flush(&mut self) -> i64 {
        if self.reply_pending() || self.request.is_empty() {
            return self.fail(EIO);
        }
        let request = std::mem::take(&mut self.request);
        let Some(cb) = self.cb.as_mut() else {
            return self.fail(ENOTSUP);
        };
        match cb(&request) {
            Ok(reply) if reply.len() <= self.callback_cap => {
                self.reply = reply;
                self.reply_at = 0;
                0
            }
            Ok(_) => self.fail(ERANGE),
            Err(code) => self.fail(code),
        }
    }

    pub(crate) fn callback_get(&mut self) -> i64 {
        match self.reply.get(self.reply_at) {
            Some(byte) => {
                self.reply_at += 1;
                i64::from(*byte)
            }
            None => i64::from(EOF),
        }
    }
}
