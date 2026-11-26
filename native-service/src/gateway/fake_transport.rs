//! Fake evaluator transport: one FLUSH, scripted reply, stepped callbacks.
use super::fake::Fake;
use super::{Transport, ops};

impl Transport for Fake {
    fn invoke(&self, ctx: i32, op: i32, value: i32) -> i64 {
        match op {
            ops::PUT => {
                self.outbound.borrow_mut().push(value as u8);
                0
            }
            ops::GET => {
                *self.gets.borrow_mut() += 1;
                self.inbound
                    .borrow_mut()
                    .pop_front()
                    .unwrap_or(ops::EOF as i32) as i64
            }
            ops::FLUSH => {
                for byte in &self.reply {
                    self.inbound.borrow_mut().push_back(*byte as i32);
                }
                self.inbound.borrow_mut().push_back(ops::EOF as i32);
                0
            }
            ops::CB_GET => self
                .cb_in
                .borrow_mut()
                .pop_front()
                .unwrap_or(ops::EOF as i32) as i64,
            ops::CB_PUT => {
                self.cb_out.borrow_mut().push(value as u8);
                0
            }
            ops::STATUS => {
                let mut steps = self.steps.borrow_mut();
                *steps += 1;
                if self.active && *steps == 1 {
                    ops::ACTIVE
                } else {
                    ops::IDLE
                }
            }
            _ => {
                let _ = ctx;
                ops::ERR_INVAL
            }
        }
    }
}
