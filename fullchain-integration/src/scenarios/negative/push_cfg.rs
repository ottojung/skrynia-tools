//! Sender configuration and one pump drive for the negative push cases.
use crate::caps::{FixedResolver, FixedRng, TestClock};
use crate::provider::FakePush;
use crate::scenarios::transport::LoopbackPush;
use sha2::{Digest, Sha256};
use std::path::Path;
use webpush_sender::Config;
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::p1::HttpP1;
use webpush_sender::pump::{Parts, Pump};
use webpush_sender::vapid::VapidKey;

fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn config(admin: &[u8; 32], sender: &[u8; 32], key: &[u8]) -> Config {
    Config {
        sender_cap: *sender,
        ns: h(admin),
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

pub fn drive(cfg: &Config, io: &HttpP1, push: &FakePush, tmp: &Path) -> String {
    let res = FixedResolver::new("push.example.test", "93.184.216.34".parse().unwrap());
    let out = LoopbackPush::new(push.ca_der.clone(), push.addr.port());
    let vk = VapidKey::from_slice(&[3u8; 32]).unwrap();
    let (rng, clock) = (FixedRng, TestClock::new());
    std::fs::create_dir_all(tmp.join("l")).unwrap();
    let mut b = [0u8; 32];
    b.copy_from_slice(&Sha256::digest(b"b"));
    let store = FileLedgerStore::new(&tmp.join("l"), &b).unwrap();
    let mut p =
        Pump::with_ledger(Parts::new(cfg, io, &out, &res, &vk, &rng, &clock), store).unwrap();
    let err = p.run_once().unwrap_err();
    format!("{err:?}/seen={}", push.seen().len())
}
