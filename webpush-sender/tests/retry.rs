mod common;

use common::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, config, frame, resolver, sub_json, vapid_key,
};
use webpush_sender::{Pump, ledger::MemoryLedgerStore, pump::Parts};

#[test]
fn transient_failures_retry_then_ack() {
    let cfg = config();
    let p1 = FakeP1::new(vec![7], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![503, 429, 204]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(1000);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(pump.run_once().unwrap().acked, 1);
    assert_eq!(out.calls.borrow().len(), 3);
    assert_eq!(clock.sleeps.borrow().len(), 2);
    assert!(p1.opcodes().contains(&22));
}

#[test]
fn out_of_order_hint_survives_lower_seq_transient_failure() {
    // F1 regression: hint 5 exhausts retries (503) while hint 6 succeeds.
    // The next run must still deliver/ack hint 5; a ledger "reuse" signal must
    // never abort the cycle and starve the remaining durable queue.
    let cfg = config();
    let p1 = FakeP1::new(vec![5, 6], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![503, 503, 503, 201, 201]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let store = MemoryLedgerStore::new();
    let parts_a = Parts::new(&cfg, &p1, &out, &r, &key, &rng, &clock);
    let mut a = Pump::with_ledger(parts_a, store.clone()).unwrap();
    assert!(a.run_once().is_err());
    assert_eq!(out.calls.borrow().len(), 4);
    let parts_b = Parts::new(&cfg, &p1, &out, &r, &key, &rng, &clock);
    let mut b = Pump::with_ledger(parts_b, store).unwrap();
    assert_eq!(b.run_once().unwrap().acked, 1);
    assert_eq!(out.calls.borrow().len(), 5);
    assert_eq!(b.run_once().unwrap().skipped, 2);
}

#[test]
fn runtime_config_errors_are_redacted() {
    common::claims::config_err_redacted();
}
