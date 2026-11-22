mod common;

use common::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};
use webpush_sender::Error;

#[test]
fn exhausted_retries_never_ack() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![500, 500, 500]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once(), Err(Error::Exhausted));
    assert!(!p1.opcodes().contains(&22));
}

#[test]
fn replay_is_skipped_without_new_sends() {
    let cfg = config();
    let p1 = FakeP1::new(vec![3], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![201, 201]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once().unwrap().delivered, 1);
    let second = pump.run_once().unwrap();
    assert_eq!(second.skipped, 1);
    assert_eq!(second.acked, 0);
    assert_eq!(out.calls.borrow().len(), 1);
}
