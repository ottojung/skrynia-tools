//! Deliver one hint. The 32-byte hint is the payload: never object data.
use super::{Error, Pump, Result, build, encrypt, parse_frame, send_with_retry, validate_url};

pub fn process_seq<P, O, R, G, C>(pump: &mut Pump<'_, P, O, R, G, C>, seq: u64) -> Result<usize>
where
    P: super::P1Transport,
    O: super::PushTransport,
    R: super::Resolver,
    G: super::Rng,
    C: super::Clock,
{
    let hint = pump.p1.hint_read(seq)?;
    if hint.len() != 32 {
        return Err(Error::Wire("hint"));
    }
    let frame = pump.p1.sub_fetch()?;
    let subs = parse_frame(&frame)?;
    let mut sent = 0;
    for sub in &subs {
        let target = validate_url(&sub.endpoint, pump.resolver)?;
        let body = encrypt(&hint, sub, pump.rng)?;
        let exp = pump.clock.now_ms() / 1000 + u64::from(pump.cfg.ttl_secs);
        let req = build(
            &target,
            body,
            pump.key,
            exp,
            &pump.cfg.vapid_sub,
            pump.cfg.ttl_secs,
            pump.cfg.topic,
        )?;
        send_with_retry(
            &req,
            pump.out,
            pump.clock,
            &pump.backoff,
            pump.cfg.max_attempts,
        )?;
        sent += 1;
    }
    Ok(sent)
}
