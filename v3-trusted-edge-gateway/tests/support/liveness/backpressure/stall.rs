use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const PROGRESS: usize = 256 * 1024;
const STABLE: Duration = Duration::from_millis(300);

/// Prove the upstream write is stalled by downstream backpressure while the
/// held request is still live. The proof (including the stability window)
/// must complete before `deadline`, never after it.
pub fn wait_backpressure(written: &AtomicUsize, total: usize, deadline: Instant) {
    let mut prev = written.load(Ordering::Relaxed);
    let mut stable_since = Instant::now();
    loop {
        let now = written.load(Ordering::Relaxed);
        let grew = now.saturating_sub(prev);
        if now > PROGRESS && grew <= PROGRESS {
            if stable_since.elapsed() >= STABLE {
                assert!(
                    Instant::now() < deadline,
                    "backpressure stall proved after request deadline: written={now}/{total}"
                );
                assert!(
                    now < total.saturating_sub(1024 * 1024),
                    "upstream must not near-complete {total} while downstream is nonreading: {now}/{total}"
                );
                return;
            }
        } else if grew > PROGRESS {
            prev = now;
            stable_since = Instant::now();
        }
        assert!(
            Instant::now() < deadline,
            "upstream did not stall under backpressure before request deadline: written={now}/{total}"
        );
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_default();
        thread::sleep(remaining.min(Duration::from_millis(100)));
    }
}
