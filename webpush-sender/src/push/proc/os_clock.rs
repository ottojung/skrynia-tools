use super::Clock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct OsClock;

impl Clock for OsClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn sleep_ms(&self, ms: u64) {
        std::thread::sleep(Duration::from_millis(ms));
    }
}
