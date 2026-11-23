use super::{FakeP1, HINT, list};
use webpush_sender::error::IoFailure;

pub fn reply(p1: &FakeP1, req: &[u8]) -> Result<Vec<u8>, IoFailure> {
    if req.first() != Some(&1) {
        return Err(IoFailure);
    }
    let body = match req.get(1).copied() {
        Some(20) => list(&p1.hints),
        Some(21) => {
            p1.hint_seen.set(true);
            HINT.to_vec()
        }
        Some(22) if p1.fail_ack.get() => return Err(IoFailure),
        Some(22) => Vec::new(),
        Some(23) if p1.hint_seen.get() => p1.alternate_subs.as_ref().unwrap_or(&p1.subs).clone(),
        Some(23) => p1.subs.clone(),
        _ => return Err(IoFailure),
    };
    Ok(body)
}
