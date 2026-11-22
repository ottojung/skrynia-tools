//! One subscription: validate, encrypt, build, send. Structural errors let the
//! caller fail closed for just this item; delivery errors withhold the ACK.
use super::{Pump, Result, Subscription, build, encrypt, send_with_retry, topic_for, validate_url};

pub fn deliver_one<P, O, R, G, C>(
    pump: &mut Pump<'_, P, O, R, G, C>,
    seq: u64,
    hint: &[u8],
    sub: &Subscription,
) -> Result<()>
where
    P: super::P1Transport,
    O: super::PushTransport,
    R: super::Resolver,
    G: super::Rng,
    C: super::Clock,
{
    let target = validate_url(&sub.endpoint, pump.resolver)?;
    let body = encrypt(hint, sub, pump.rng)?;
    let exp = pump.clock.now_ms() / 1000 + u64::from(pump.cfg.ttl_secs);
    let req = build(
        &target,
        body,
        pump.key,
        exp,
        &pump.cfg.vapid_sub,
        pump.cfg.ttl_secs,
        topic_for(&pump.cfg.topic, seq, hint),
    )?;
    send_with_retry(
        &req,
        pump.out,
        pump.clock,
        &pump.backoff,
        pump.cfg.max_attempts,
    )
}
