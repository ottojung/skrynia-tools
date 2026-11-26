//! Deterministic fake evaluator state for the gateway tests.
use std::cell::RefCell;
use std::collections::VecDeque;

pub struct Fake {
    pub inbound: RefCell<VecDeque<i32>>,
    pub outbound: RefCell<Vec<u8>>,
    pub cb_in: RefCell<VecDeque<i32>>,
    pub cb_out: RefCell<Vec<u8>>,
    pub gets: RefCell<u32>,
    pub steps: RefCell<u32>,
    pub active: bool,
    pub reply: Vec<u8>,
}

impl Fake {
    pub fn new(reply: Vec<u8>) -> Self {
        Self {
            inbound: RefCell::new(VecDeque::new()),
            outbound: RefCell::new(Vec::new()),
            cb_in: RefCell::new(VecDeque::new()),
            cb_out: RefCell::new(Vec::new()),
            gets: RefCell::new(0),
            steps: RefCell::new(0),
            active: false,
            reply,
        }
    }
}
