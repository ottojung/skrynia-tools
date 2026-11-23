//! Real native sender consumes high-bit fallback and decrypts WebPush payload.
use super::{ctx::Ctx, wire as w};
use crate::{FixedResolver, FixedRng, LoopbackPush, TestClock, check};
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::pump::{Parts, Pump};
use webpush_sender::{Config, vapid::VapidKey};
pub fn deliver(c: &Ctx) {
    let cfg = Config {
        sender_cap: w::sender(), ns: w::ns(), key: w::KEY.to_vec(),
        max_payload: 4096, max_attempts: 3, base_backoff_ms: 10,
        max_backoff_ms: 100, ttl_secs: 60, topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".into(), ledger_epoch: 0,
    };
    let res = FixedResolver::new("push.example.test", "93.184.216.34".parse().unwrap());
    let out = LoopbackPush::new(c.push.ca_der.clone(), c.push.addr.port());
    let vk = VapidKey::from_slice(&[3u8; 32]).unwrap();
    let (rng, clock) = (FixedRng, TestClock::new());
    let dir = c.tmp.join("ledger");
    std::fs::create_dir_all(&dir).unwrap();
    let store = FileLedgerStore::new(&dir, &w::h(b"ordinary-ledger")).unwrap();
    let mut pump = Pump::with_ledger(
        Parts::new(&cfg, &c.io, &out, &res, &vk, &rng, &clock), store,
    ).unwrap();
    let rep = pump.run_once().unwrap();
    assert_eq!((rep.hints, rep.delivered, rep.acked), (1, 1, 1));
    let seen = c.push.seen();
    assert_eq!(seen.len(), 1);
    let parsed = check(&seen[0], &c.sub_keys, "https://push.example.test").unwrap();
    assert_eq!(parsed.plaintext, [0u8; 32]);
    assert_eq!(w::list(c).len(), 256, "legacy hints reappear after fallback ACK");
    let old = pump.run_once().expect("sender drains older hash hints");
    assert_eq!((old.hints, old.delivered, old.acked), (256, 256, 256));
    let observed = c.push.seen();
    assert_eq!(observed.len(), 257, "old sequence drain delivered exactly once");
    let first = check(&observed[1], &c.sub_keys, "https://push.example.test").unwrap();
    assert_eq!(first.plaintext, w::h(&0u16.to_le_bytes()));
    assert!(w::list(c).is_empty(), "all older hints ACKed");
    drop(pump);
    let restored = FileLedgerStore::new(&dir, &w::h(b"ordinary-ledger")).unwrap();
    let mut restart = Pump::with_ledger(
        Parts::new(&cfg, &c.io, &out, &res, &vk, &rng, &clock), restored,
    ).expect("ledger must reopen after high-bit then lower seq delivery");
    assert_eq!(restart.run_once().unwrap().hints, 0);
}
