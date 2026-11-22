use super::*;
use webpush_sender::Pump;

#[test]
fn retry_after_is_minimum_for_http_429() {
    let cfg = config();
    let p1 = FakeP1::new(vec![8], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![429, 201]);
    out.retry_after_ms.set(Some(5_000));
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(pump.run_once().unwrap().acked, 1);
    assert_eq!(*clock.sleeps.borrow(), [5_000]);
}

#[test]
fn attacker_supplied_retry_after_capped_for_http_503() {
    let cfg = config();
    let p1 = FakeP1::new(vec![9], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![503, 429, 201]);
    out.retry_after_ms.set(Some(u64::MAX));
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(pump.run_once().unwrap().acked, 1);
    assert_eq!(*clock.sleeps.borrow(), [60_000, 60_000]);
}

#[test]
fn nontransient_http_400_never_obeys_retry_after() {
    let cfg = config();
    let p1 = FakeP1::new(vec![10], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![400]);
    out.retry_after_ms.set(Some(60_000));
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert!(pump.run_once().is_err());
    assert!(clock.sleeps.borrow().is_empty());
    assert!(!p1.opcodes().contains(&22));
}
