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
