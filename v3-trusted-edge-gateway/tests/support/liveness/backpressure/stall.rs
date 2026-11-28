use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

pub fn wait_backpressure(written: &AtomicUsize, total: usize) {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(5) {
        if written.load(Ordering::Relaxed) > 256 * 1024 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let start = written.load(Ordering::Relaxed);
    assert!(
        start > 256 * 1024,
        "upstream must produce data before backpressure check: {start}"
    );
    let mut prev = start;
    let mut stable_since = Instant::now();
    loop {
        thread::sleep(Duration::from_millis(100));
        let now = written.load(Ordering::Relaxed);
        if now <= prev.saturating_add(256 * 1024) {
            if stable_since.elapsed() >= Duration::from_millis(300) {
                assert!(
                    now < total.saturating_sub(1024 * 1024),
                    "upstream must not near-complete 16MiB while downstream is nonreading: {now}/{total}"
                );
                return;
            }
        } else {
            prev = now;
            stable_since = Instant::now();
        }
        assert!(
            t.elapsed() < Duration::from_millis(1700),
            "upstream did not stall under backpressure before request deadline: start={start} now={now} total={total}"
        );
    }
}
