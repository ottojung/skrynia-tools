//! Retry only transient failures; never retry auth/SSRF/malformed responses.
use super::{Backoff, Clock, Error, PushRequest, PushTransport, Result};

pub fn send_with_retry(
    req: &PushRequest,
    out: &impl PushTransport,
    clock: &impl Clock,
    backoff: &Backoff,
    max_attempts: u8,
) -> Result<()> {
    let mut attempt = 0u8;
    loop {
        match out.send(req) {
            Ok(reply) if (200..300).contains(&reply.status) => return Ok(()),
            Ok(reply) if reply.status == 429 || reply.status >= 500 => {}
            Ok(_) => return Err(Error::Transport("status")),
            Err(_) => {}
        }
        attempt += 1;
        if attempt >= max_attempts {
            return Err(Error::Exhausted);
        }
        let jitter = (clock.now_ms() & 0xff) as u8;
        clock.sleep_ms(backoff.delay(attempt, jitter));
    }
}
