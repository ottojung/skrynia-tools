use super::fakes::{FakeP1, FakeResolver, FixedRng, ManualClock, ScriptedTransport};
use webpush_sender::{Config, Pump, vapid::VapidKey};

pub fn build<'a>(
    cfg: &'a Config,
    p1: &'a FakeP1,
    out: &'a ScriptedTransport,
    r: &'a FakeResolver,
    key: &'a VapidKey,
    rng: &'a FixedRng,
    clock: &'a ManualClock,
) -> Pump<'a, FakeP1, ScriptedTransport, FakeResolver, FixedRng, ManualClock> {
    Pump::new(cfg, p1, out, r, key, rng, clock).unwrap()
}
