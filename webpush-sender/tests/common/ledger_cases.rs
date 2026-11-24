//! End-to-end crash/replay cases against the injected durable ledger.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, config, frame, resolver, sub_json, vapid_key,
};
use webpush_sender::pump::Parts;
use webpush_sender::{Error, Pump, ledger::MemoryLedgerStore};

#[test]
fn quota_before_ack_withholds_ack() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![201]);
    let store = MemoryLedgerStore::new();
    let faults = store.faults();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump =
        Pump::with_ledger(Parts::new(&cfg, &p1, &out, &r, &key, &rng, &clock), store).unwrap();
    faults.borrow_mut().save = Some(Error::Quota);
    assert_eq!(pump.run_once(), Err(Error::Quota));
    assert!(!p1.opcodes().contains(&22));
    assert_eq!(out.calls.borrow().len(), 1);
}

#[test]
fn lost_ack_replayed_without_resend() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/p")]));
    p1.fail_ack.set(true);
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![201]);
    let store = MemoryLedgerStore::new();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut a = Pump::with_ledger(
        Parts::new(&cfg, &p1, &out, &r, &key, &rng, &clock),
        store.clone(),
    )
    .unwrap();
    assert!(a.run_once().is_err());
    assert_eq!(out.calls.borrow().len(), 1);
    p1.fail_ack.set(false);
    let mut b =
        Pump::with_ledger(Parts::new(&cfg, &p1, &out, &r, &key, &rng, &clock), store).unwrap();
    assert_eq!(b.run_once().unwrap().acked, 1);
    assert_eq!(b.run_once().unwrap().skipped, 1);
    assert_eq!(out.calls.borrow().len(), 1);
}
