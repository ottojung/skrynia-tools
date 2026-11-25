//! Backoff policy and transient-only retry loop.
use super::{Clock, Error, PushRequest, PushTransport, Result};

/// Bounded exponential backoff with a small jitter, capped at `max_ms`.
pub struct Backoff {
    pub base_ms: u64,
    pub max_ms: u64,
}

impl Backoff {
    pub fn delay(&self, attempt: u8, jitter: u8) -> u64 {
        let shift = u32::from(attempt.min(20));
        let doubled = self.base_ms.saturating_mul(1u64 << shift);
        let base = doubled.min(self.max_ms);
        let noise = u64::from(jitter) % (base / 4 + 1);
        base.saturating_add(noise).min(self.max_ms)
    }
}

/// Retry only transient failures. 404/410 are terminal per subscription
/// (Gone) so one expired endpoint cannot block ACK for other recipients.
/// 401/403/other permanent status fail closed.
pub fn send_with_retry(
    req: &PushRequest,
    out: &impl PushTransport,
    clock: &impl Clock,
    backoff: &Backoff,
    max_attempts: u8,
) -> Result<()> {
    let mut attempt = 0u8;
    loop {
        let server_delay_ms = match out.send(req) {
            Ok(reply) if (200..300).contains(&reply.status) => return Ok(()),
            Ok(reply) if reply.status == 404 || reply.status == 410 => {
                return Err(Error::Transport("gone"));
            }
            Ok(reply) if reply.status == 401 || reply.status == 403 => {
                return Err(Error::Auth("push"));
            }
            Ok(reply) if reply.status == 429 || reply.status >= 500 => {
                reply.retry_after_ms.unwrap_or(0).min(60_000)
            }
            Ok(_) => return Err(Error::Transport("status")),
            Err(_) => 0,
        };
        attempt += 1;
        if attempt >= max_attempts {
            return Err(Error::Exhausted);
        }
        let jitter = (clock.now_ms() & 0xff) as u8;
        clock.sleep_ms(backoff.delay(attempt, jitter).max(server_delay_ms));
    }
}
