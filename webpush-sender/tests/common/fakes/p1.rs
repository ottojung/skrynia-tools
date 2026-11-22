//! In-memory P1 endpoint. Records opcode order for re-read assertions.
use super::{HINT, list, res};
use std::cell::{Cell, RefCell};
use webpush_sender::error::IoFailure;
use webpush_sender::p1::P1Transport;

pub struct FakeP1 {
    pub hints: Vec<u64>,
    pub subs: Vec<u8>,
    pub alternate_subs: Option<Vec<u8>>,
    pub calls: RefCell<Vec<Vec<u8>>>,
    hint_seen: Cell<bool>,
}

impl FakeP1 {
    pub fn new(hints: Vec<u64>, subs: Vec<u8>) -> Self {
        Self {
            hints,
            subs,
            alternate_subs: None,
            calls: RefCell::new(Vec::new()),
            hint_seen: Cell::new(false),
        }
    }

    pub fn opcodes(&self) -> Vec<u8> {
        self.calls.borrow().iter().map(|c| c[0]).collect()
    }
}

impl P1Transport for FakeP1 {
    fn call(&self, req: &[u8]) -> core::result::Result<Vec<u8>, IoFailure> {
        self.calls.borrow_mut().push(req.to_vec());
        let body = match req.first().copied() {
            Some(20) => list(&self.hints),
            Some(21) => {
                self.hint_seen.set(true);
                HINT.to_vec()
            }
            Some(22) => Vec::new(),
            Some(23) if self.hint_seen.get() => {
                self.alternate_subs.as_ref().unwrap_or(&self.subs).clone()
            }
            Some(23) => self.subs.clone(),
            _ => return Err(IoFailure),
        };
        Ok(res(0, &body))
    }
}
