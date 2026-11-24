//! Pump state. Every side effect is an injected capability reference.
use super::{
    Backoff, Client, Clock, Config, Ledger, MemoryLedgerStore, P1Transport, Parts, PushTransport,
    Resolver, Result, Rng, VapidKey,
};

pub struct Pump<'a, P: P1Transport, O: PushTransport, R: Resolver, G: Rng, C: Clock> {
    pub cfg: &'a Config,
    pub p1: Client<'a, P>,
    pub out: &'a O,
    pub resolver: &'a R,
    pub key: &'a VapidKey,
    pub rng: &'a G,
    pub clock: &'a C,
    pub ledger: Ledger,
    pub backoff: Backoff,
}

impl<'a, P: P1Transport, O: PushTransport, R: Resolver, G: Rng, C: Clock> Pump<'a, P, O, R, G, C> {
    /// Hermetic test constructor: in-memory store, not durable across runs.
    pub fn new(
        cfg: &'a Config,
        p1_io: &'a P,
        out: &'a O,
        resolver: &'a R,
        key: &'a VapidKey,
        rng: &'a G,
        clock: &'a C,
    ) -> Result<Self> {
        Self::with_ledger(
            Parts::new(cfg, p1_io, out, resolver, key, rng, clock),
            MemoryLedgerStore::new(),
        )
    }
}
