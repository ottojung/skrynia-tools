use super::super::transport::LoopbackPush;
use super::ctx::Ctx;
use crate::caps::{FixedResolver, FixedRng, TestClock};
use sha2::{Digest, Sha256};
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::pump::{Parts, Pump, Report};
use webpush_sender::{Config, vapid::VapidKey};

pub fn deliver(c: &mut Ctx) -> (Report, usize) {
    run(c, "ledger")
}
pub fn deliver_again(c: &mut Ctx) -> (Report, usize) {
    run(c, "ledger")
}
fn run(c: &mut Ctx, ledger: &str) -> (Report, usize) {
    let cfg = Config {
        sender_cap: c.sender_cap,
        ns: c.ns,
        key: c.key.clone(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".to_owned(),
        ledger_epoch: 0,
    };
    let res = FixedResolver::new("push.example.test", "93.184.216.34".parse().unwrap());
    let out = LoopbackPush::new(c.push.ca_der.clone(), c.push.addr.port());
    let key = VapidKey::from_slice(&[3u8; 32]).unwrap();
    let (rng, clock) = (FixedRng, TestClock::new());
    let dir = c.tmp.join(ledger);
    std::fs::create_dir_all(&dir).unwrap();
    let binding = Sha256::digest(b"fullchain-ledger-binding-v1");
    let mut b = [0u8; 32];
    b.copy_from_slice(&binding);
    let store = FileLedgerStore::new(&dir, &b).unwrap();
    let mut p = Pump::with_ledger(
        Parts::new(&cfg, &c.io, &out, &res, &key, &rng, &clock),
        store,
    )
    .unwrap();
    let report = p.run_once().unwrap();
    (report, out.sent.lock().unwrap().len())
}
