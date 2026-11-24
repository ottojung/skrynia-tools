//! Bundled injected capabilities for explicit pump construction.
use super::{Clock, Config, P1Transport, PushTransport, Resolver, Rng, VapidKey};

pub struct Parts<'a, P: P1Transport, O: PushTransport, R: Resolver, G: Rng, C: Clock> {
    pub cfg: &'a Config,
    pub p1: &'a P,
    pub out: &'a O,
    pub resolver: &'a R,
    pub key: &'a VapidKey,
    pub rng: &'a G,
    pub clock: &'a C,
}

impl<'a, P: P1Transport, O: PushTransport, R: Resolver, G: Rng, C: Clock> Parts<'a, P, O, R, G, C> {
    pub fn new(
        cfg: &'a Config,
        p1: &'a P,
        out: &'a O,
        resolver: &'a R,
        key: &'a VapidKey,
        rng: &'a G,
        clock: &'a C,
    ) -> Self {
        Self {
            cfg,
            p1,
            out,
            resolver,
            key,
            rng,
            clock,
        }
    }
}
