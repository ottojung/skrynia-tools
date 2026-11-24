//! Production constructor. All capabilities are explicit; the durable store
//! is injected and cannot be replaced by an ambient or shared path.
use super::{
    Backoff, Client, Clock, Ledger, LedgerStore, P1Transport, Parts, Pump, PushTransport, Resolver,
    Result, Rng, check_scope,
};

impl<'a, P: P1Transport, O: PushTransport, R: Resolver, G: Rng, C: Clock> Pump<'a, P, O, R, G, C> {
    pub fn with_ledger<S: LedgerStore + 'static>(
        parts: Parts<'a, P, O, R, G, C>,
        store: S,
    ) -> Result<Self> {
        parts.cfg.validate()?;
        check_scope(&parts.cfg.sender_cap, &parts.cfg.ns, &parts.cfg.key)?;
        let ledger = Ledger::open(store, parts.cfg)?;
        Ok(Self {
            cfg: parts.cfg,
            p1: Client::new(parts.cfg, parts.p1)?,
            out: parts.out,
            resolver: parts.resolver,
            key: parts.key,
            rng: parts.rng,
            clock: parts.clock,
            ledger,
            backoff: Backoff {
                base_ms: parts.cfg.base_backoff_ms,
                max_ms: parts.cfg.max_backoff_ms,
            },
        })
    }
}
