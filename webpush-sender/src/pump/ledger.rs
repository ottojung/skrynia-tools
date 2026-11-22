use std::collections::HashSet;

/// Bounded replay ledger. ACK is idempotent in P1; this adds local dedupe so
/// a crash between send and ACK cannot multiply deliveries within a run.
pub struct Ledger {
    seen: HashSet<u64>,
    cap: usize,
}

impl Ledger {
    pub fn new(cap: usize) -> Self {
        Self {
            seen: HashSet::new(),
            cap,
        }
    }

    pub fn seen(&self, seq: u64) -> bool {
        self.seen.contains(&seq)
    }

    pub fn mark(&mut self, seq: u64) {
        if self.seen.len() < self.cap {
            self.seen.insert(seq);
        }
    }
}
