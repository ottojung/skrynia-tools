//! Grant binding and bounded record model. Raw capability bytes never leave
//! this module; only a one-way digest is used as the file identity.
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Delivered = 1,
    Acked = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub seq: u64,
    pub state: State,
}

/// Bind to EXACT P1 sender grant (cap, ns, key) plus an operator epoch.
/// A rotated grant or a bumped epoch yields a different binding, so no ledger
/// state can poison an unrelated sender grant or a migrated hint sequence.
pub fn binding(cap: &[u8], ns: &[u8], key: &[u8], epoch: u64) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"skrynia-webpush-ledger-v1");
    h.update(cap);
    h.update(ns);
    h.update(key);
    h.update(epoch.to_le_bytes());
    h.finalize().into()
}

pub fn digest(src: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"skrynia-ledger-digest-v1");
    h.update(src);
    h.finalize().into()
}

#[cfg(test)]
pub(crate) fn test_cfg() -> crate::Config {
    crate::Config {
        sender_cap: [7u8; 32],
        ns: [9u8; 32],
        key: b"k".to_vec(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 1,
        max_backoff_ms: 2,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".to_owned(),
        ledger_epoch: 0,
    }
}
