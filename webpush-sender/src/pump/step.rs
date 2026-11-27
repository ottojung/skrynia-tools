//! Deliver one hint. The 32-byte hint is the payload: never object data.
use super::{Error, Pump, Result, deliver_one, keep_worse, parse_frame_isolated};

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
    let subs = parse_frame_isolated(&frame)?;
    let mut sent = 0;
    let mut failure = None;
    for sub in &subs {
        match deliver_one(pump, seq, &hint, sub) {
            Ok(()) => sent += 1,
            // 404/410: endpoint gone; terminal per item, no prune, do not block ACK.
            Err(Error::Transport("gone")) => {}
            // Genuine delivery failure: never ACK this hint, but keep trying
            // the remaining subscriptions and the remaining queue entries.
            Err(e @ (Error::Auth(_) | Error::Exhausted | Error::Transport(_))) => {
                keep_worse(&mut failure, e)
            }
            // Malformed or unsafe subscription: fail closed for this item only.
            Err(_) => {}
        }
    }
    match failure {
        Some(e) => Err(e),
        None => Ok(sent),
    }
}
