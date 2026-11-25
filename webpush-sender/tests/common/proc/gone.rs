//! Terminal Gone and transient failure handling with the fake transport.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};

pub fn gone_and_success() {
    let cfg = config();
    let p1 = FakeP1::new(
        vec![1],
        frame(&[
            sub_json("https://gone.example/p"),
            sub_json("https://ok.example/p"),
        ]),
    );
    let r = resolver(&[
        ("gone.example", "93.184.216.34"),
        ("ok.example", "93.184.216.35"),
    ]);
    let out = ScriptedTransport::new(vec![404, 201]);
    let key = vapid_key();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    let report = pump.run_once().unwrap();
    assert_eq!(report.delivered, 1);
    assert_eq!(report.acked, 1);
    assert_eq!(out.calls.borrow().len(), 2, "good recipient not repeated");
    assert!(p1.opcodes().contains(&22));
}

pub fn transient_no_ack() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://ok.example/p")]));
    let r = resolver(&[("ok.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![500, 500, 500]);
    let key = vapid_key();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert!(pump.run_once().is_err());
    assert_eq!(out.calls.borrow().len(), 3);
    assert!(!p1.opcodes().contains(&22));
}
