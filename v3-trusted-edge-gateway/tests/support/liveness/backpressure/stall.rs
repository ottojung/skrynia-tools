use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use std::{fs, thread};

pub fn wait_stall(path: &std::path::Path, wr: &std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(5) {
        let n = fs::read_to_string(path)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if n > 4 && wr.load(Ordering::Relaxed) > 256 * 1024 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let p1: usize = fs::read_to_string(path)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    thread::sleep(Duration::from_millis(400));
    let p2: usize = fs::read_to_string(path)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    assert!(
        p2 <= p1 + 2,
        "poll counter must stall under backpressure: {p1}->{p2}"
    );
}
