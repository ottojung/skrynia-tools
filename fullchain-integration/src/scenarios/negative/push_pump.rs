//! Deterministic Pump config, VAPID key and ledger binding for negative runs.
use sha2::{Digest, Sha256};
use std::path::Path;
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::{Config, vapid::VapidKey};

pub fn config(sender: [u8; 32], ns: [u8; 32], key: &[u8]) -> Config {
    Config {
        sender_cap: sender,
        ns,
        key: key.to_vec(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".into(),
        ledger_epoch: 0,
    }
}
pub fn vapid_key() -> VapidKey {
    VapidKey::from_slice(&[3u8; 32]).unwrap()
}
pub fn ledger(tmp: &Path) -> FileLedgerStore {
    std::fs::create_dir_all(tmp.join("l")).unwrap();
    let mut b = [0u8; 32];
    b.copy_from_slice(&Sha256::digest(b"b"));
    FileLedgerStore::new(&tmp.join("l"), &b).unwrap()
}
