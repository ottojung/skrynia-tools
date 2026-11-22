use std::cell::{Cell, RefCell};
use webpush_sender::pump::Clock;

pub struct ManualClock {
    pub now: Cell<u64>,
    pub sleeps: RefCell<Vec<u64>>,
}

impl ManualClock {
    pub fn new(now: u64) -> Self {
        Self {
            now: Cell::new(now),
            sleeps: RefCell::new(Vec::new()),
        }
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.now.get()
    }

    fn sleep_ms(&self, ms: u64) {
        self.now.set(self.now.get() + ms);
        self.sleeps.borrow_mut().push(ms);
    }
}
