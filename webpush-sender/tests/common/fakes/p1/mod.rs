mod body;
use super::{HINT, list, res};
use std::cell::{Cell, RefCell};
use webpush_sender::error::IoFailure;
use webpush_sender::p1::{P1Failure, P1Transport};

pub struct FakeP1 {
    pub hints: Vec<u64>,
    pub subs: Vec<u8>,
    pub alternate_subs: Option<Vec<u8>>,
    pub calls: RefCell<Vec<Vec<u8>>>,
    pub fail_ack: Cell<bool>,
    hint_seen: Cell<bool>,
}
impl FakeP1 {
    pub fn new(hints: Vec<u64>, subs: Vec<u8>) -> Self {
        Self {
            hints,
            subs,
            alternate_subs: None,
            calls: RefCell::new(Vec::new()),
            fail_ack: Cell::new(false),
            hint_seen: Cell::new(false),
        }
    }
    pub fn opcodes(&self) -> Vec<u8> {
        self.calls.borrow().iter().filter_map(|c| op(c)).collect()
    }
}
fn op(c: &[u8]) -> Option<u8> {
    if c.first() == Some(&1) {
        c.get(1).copied()
    } else {
        None
    }
}
impl P1Transport for FakeP1 {
    fn call(&self, req: &[u8]) -> core::result::Result<Vec<u8>, P1Failure> {
        self.calls.borrow_mut().push(req.to_vec());
        let body = body::reply(self, req).map_err(|_| P1Failure::Io)?;
        Ok(res(0, &body))
    }
}
