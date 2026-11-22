//! Regressions for the three hard NO-GO findings at pinned base 0c340b.
mod common;

use common::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, jwt_aud, one, sub_json,
    vapid_key,
};

#[test]
fn vapid_aud_is_rfc6454_origin() {
    let (cfg, key) = (config(), vapid_key());
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/p")]));
    let r = one("93.184.216.34");
    let out = ScriptedTransport::new(vec![201]);
    let (rng, clock) = (FixedRng { seed: [4u8; 32] }, ManualClock::new(0));
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    pump.run_once().unwrap();
    let aud = jwt_aud(&out.calls.borrow()[0].authorization);
    assert_eq!(aud, "https://push.example");
    assert_ne!(aud, "https://push.example:443");
}

#[test]
fn bad_subscription_does_not_starve_valid_one() {
    let (cfg, key) = (config(), vapid_key());
    let bad = b"not-json".to_vec();
    let p1 = FakeP1::new(vec![9], frame(&[bad, sub_json("https://push.example/p")]));
    let r = one("93.184.216.34");
    let out = ScriptedTransport::new(vec![201]);
    let (rng, clock) = (FixedRng { seed: [4u8; 32] }, ManualClock::new(0));
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    let report = pump.run_once().unwrap();
    assert_eq!((report.delivered, report.acked), (1, 1));
    assert_eq!((report.hints, report.skipped), (1, 0));
    assert_eq!(out.calls.borrow().len(), 1);
    assert!(p1.opcodes().contains(&22));
}

#[test]
fn embedded_v6_and_zero_eighth_never_send() {
    let (cfg, key) = (config(), vapid_key());
    for bad in ["0.1.2.3", "64:ff9b::0a00:1", "2002:0a00:1::", "2001::1"] {
        let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/p")]));
        let r = one(bad);
        let out = ScriptedTransport::new(vec![201]);
        let (rng, clock) = (FixedRng { seed: [4u8; 32] }, ManualClock::new(0));
        let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
        assert_eq!(pump.run_once().unwrap().delivered, 0, "{bad}");
        assert!(out.calls.borrow().is_empty(), "{bad}");
    }
}
