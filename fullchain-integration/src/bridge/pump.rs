//! One real sender cycle against the caller's P1 and controlled HTTPS endpoint.
use super::config::Config;
use crate::{FixedResolver, FixedRng, LoopbackPush, TestClock};
use sha2::{Digest, Sha256};
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::p1::HttpP1;
use webpush_sender::pump::{Parts, Pump, Report};
use webpush_sender::{Config as SenderConfig, vapid::VapidKey};

pub fn run(cfg: &Config, io: &HttpP1, push: &LoopbackPush) -> Report {
    let sc = SenderConfig {
        sender_cap: cfg.sender_cap,
        ns: cfg.ns,
        key: cfg.key.clone(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".to_owned(),
        ledger_epoch: 0,
    };
    let resolver = FixedResolver::new("push.example.test", "93.184.216.34".parse().unwrap());
    let key = VapidKey::from_slice(&[3u8; 32]).expect("vapid test key");
    let (rng, clock) = (FixedRng, TestClock::new());
    let b = Sha256::digest(b"fullchain-ledger-binding-v1");
    let mut binding = [0u8; 32];
    binding.copy_from_slice(&b);
    let store = FileLedgerStore::new(&cfg.ledger, &binding).expect("durable ledger");
    let parts = Parts::new(&sc, io, push, &resolver, &key, &rng, &clock);
    let mut p = Pump::with_ledger(parts, store).expect("pump");
    p.run_once().expect("one sender cycle")
}
