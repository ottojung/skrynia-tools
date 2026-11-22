mod common;

use common::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, config, frame, resolver, sub_json, vapid_key,
};
use webpush_sender::Pump;

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
