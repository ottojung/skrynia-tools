#[derive(Clone, Debug)]
pub struct Policy {
    pub max_failures: u32,
    pub base_ms: u64,
    pub max_ms: u64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            max_failures: 12,
            base_ms: 1000,
            max_ms: 60 * 60 * 1000,
        }
    }
}
impl Policy {
    pub fn backoff(&self, attempts: u32) -> u64 {
        let shift = attempts.saturating_sub(1).min(30);
        let factor = 1u64 << shift;
        (self.base_ms.saturating_mul(factor)).min(self.max_ms)
    }
}
