use super::os_clock::OsClock;
use super::os_rng::OsRng;
use super::policy::{backoff_ms, fatal};
use super::{Clock, HttpsTransport, P1Transport, Pump, Resolver, Result};

pub(super) fn run_loop<P, R>(
    pump: &mut Pump<'_, P, HttpsTransport, R, OsRng, OsClock>,
    poll_ms: u64,
    max_backoff_ms: u64,
) -> Result<()>
where
    P: P1Transport,
    R: Resolver,
{
    let clock = OsClock;
    let mut streak = 0u32;
    loop {
        match pump.run_once() {
            Ok(r) => {
                streak = 0;
                eprintln!(
                    "cycle ok hints={} delivered={} acked={} skipped={}",
                    r.hints, r.delivered, r.acked, r.skipped
                );
                clock.sleep_ms(poll_ms);
            }
            Err(e) => {
                if fatal(&e) {
                    return Err(e);
                }
                streak = streak.saturating_add(1);
                let wait = backoff_ms(poll_ms, max_backoff_ms, streak);
                eprintln!("cycle err class={:?} backoff_ms={}", e, wait);
                clock.sleep_ms(wait);
            }
        }
    }
}
