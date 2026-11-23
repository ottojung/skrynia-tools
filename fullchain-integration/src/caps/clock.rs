//! virtual clock: sleeps are recorded, never performed.
use std::sync::Mutex;
use webpush_sender::pump::Clock;

pub struct TestClock {
    now: Mutex<u64>,
    pub sleeps: Mutex<Vec<u64>>,
}

impl TestClock {
    pub fn new() -> Self {
        Self {
            now: Mutex::new(1_700_000_000_000),
            sleeps: Mutex::new(Vec::new()),
        }
    }
}

impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        *self.now.lock().unwrap()
    }
    fn sleep_ms(&self, ms: u64) {
        *self.now.lock().unwrap() += ms;
        self.sleeps.lock().unwrap().push(ms);
    }
}

impl Default for TestClock {
    fn default() -> Self {
        Self::new()
    }
}
