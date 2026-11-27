//! Bounded request frames and independent reply EOF.
use super::{Ctx, code};

impl Ctx {
    pub fn put(&mut self, value: i32) -> i64 {
        if self.reply_active {
            return -3;
        }
        if self.frame_failed {
            return -3;
        }
        if !(0..=255).contains(&value) || self.frame.len() >= code::MAX_FRAME {
            self.frame.clear();
            self.frame_failed = true;
            return -3;
        }
        self.frame.push(value as u8);
        0
    }
}
