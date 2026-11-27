//! Permanent edge/push auth rejection is fatal and never retried.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};

pub fn auth_rejected_once() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://ok.example/p")]));
    let r = resolver(&[("ok.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![403]);
    let key = vapid_key();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once(), Err(webpush_sender::Error::Auth("push")));
    assert_eq!(out.calls.borrow().len(), 1, "permanent auth not retried");
    assert!(!p1.opcodes().contains(&22));
}

/// Inside send_with_retry an earlier transient response must not mask a
/// permanent auth that arrives on a later attempt: the classified Auth wins.
pub fn retry_transient_then_auth_is_auth() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://a.example/p")]));
    let r = resolver(&[("a.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![503, 401]);
    let key = vapid_key();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once(), Err(webpush_sender::Error::Auth("push")));
    assert_eq!(out.calls.borrow().len(), 2, "401 after 503 stops retrying");
    assert!(!p1.opcodes().contains(&22));
}

/// A fatal auth on an earlier seq stops the cycle; later transient failures
/// are never reached or retried.
pub fn auth_stops_before_later_seq() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1, 2], frame(&[sub_json("https://a.example/p")]));
    let r = resolver(&[("a.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![401, 500, 500, 500]);
    let key = vapid_key();
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once(), Err(webpush_sender::Error::Auth("push")));
    assert_eq!(out.calls.borrow().len(), 1);
    assert!(!p1.opcodes().contains(&22));
}
