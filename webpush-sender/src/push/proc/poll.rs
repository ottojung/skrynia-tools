use super::os_clock::OsClock;
use super::os_rng::OsRng;
use super::{Clock, HttpsTransport, P1Transport, Pump, Resolver};

pub(super) fn run_loop<P, R>(
    pump: &mut Pump<'_, P, HttpsTransport, R, OsRng, OsClock>,
    poll_ms: u64,
) where
    P: P1Transport,
    R: Resolver,
{
    let clock = OsClock;
    loop {
        match pump.run_once() {
            Ok(r) => eprintln!(
                "cycle ok hints={} delivered={} acked={} skipped={}",
                r.hints, r.delivered, r.acked, r.skipped
            ),
            Err(e) => eprintln!("cycle err class={:?}", e),
        }
        clock.sleep_ms(poll_ms);
    }
}
