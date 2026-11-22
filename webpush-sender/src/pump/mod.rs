//! Bounded pump: hint -> authoritative re-read -> deliver -> ACK.
mod clock;
mod cycle;
mod deliver;
mod ledger;
mod report;
mod retry;
mod run;
mod seqs;
mod step;

pub use clock::Clock;
pub(super) use deliver::deliver_one;
pub use ledger::Ledger;
pub use report::Report;
pub use retry::Backoff;
pub(super) use retry::send_with_retry;
pub use run::Pump;
pub(super) use step::process_seq;

pub(super) use super::config::{Config, MAX_HINTS};
pub(super) use super::error::{Error, Result};
pub(super) use super::p1::{Client, P1Transport};
pub(super) use super::push::{
    HttpsTransport, PushRequest, PushTransport, Rng, build, encrypt, topic_for,
};
pub(super) use super::sub::{
    Resolver, Subscription, check_scope, parse_frame_isolated, validate_url,
};
pub(super) use super::vapid::VapidKey;

/// Opt-in production binding of the generic Pump to the real pinned-IP HTTPS
/// adapter. The caller owns the transport so its lifetime is explicit; no
/// operator policy, token or injected CA is embedded in the pump.
impl<'a, P: P1Transport, R: Resolver, G: Rng, C: Clock> Pump<'a, P, HttpsTransport, R, G, C> {
    pub fn with_https_transport(
        cfg: &'a Config,
        p1_io: &'a P,
        out: &'a HttpsTransport,
        resolver: &'a R,
        key: &'a VapidKey,
        rng: &'a G,
        clock: &'a C,
    ) -> Result<Self> {
        Self::new(cfg, p1_io, out, resolver, key, rng, clock)
    }
}
