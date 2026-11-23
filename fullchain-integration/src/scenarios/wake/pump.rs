//! Actual HTTPS provider with ES256 VAPID and aes128gcm proof for shared wake.
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
    let store = FileLedgerStore::new(&dir, &w::h(b"wake-ledger")).unwrap();
    let mut pump = Pump::with_ledger(
        Parts::new(&cfg, &c.io, &out, &res, &vk, &rng, &clock), store,
    ).unwrap();
    let rep = pump.run_once().unwrap();
    assert_eq!((rep.hints, rep.delivered, rep.acked), (1, 1, 1));
    let seen = c.push.seen();
    assert_eq!(seen.len(), 1);
    let v = check(&seen[0], &c.sub_keys, "https://push.example.test").unwrap();
    assert_eq!(v.plaintext, [0u8; 32]);
    assert!(w::list(c).is_empty());
}
