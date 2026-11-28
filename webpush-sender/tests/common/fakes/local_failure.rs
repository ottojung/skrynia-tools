//! Transient injected-capability failures must withhold the ACK.
use super::{
    FakeP1, FakeResolver, FixedRng, HINT, ManualClock, ScriptedTransport, config, frame, resolver,
    sub_json, vapid_key,
};
use std::collections::HashMap;
use webpush_sender::{Error, Pump};

fn no_ack(p1: &FakeP1) -> bool {
    !p1.opcodes().contains(&22)
}

#[test]
fn dns_io_failure_withholds_ack() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/push")]));
    // Missing resolver entry is an injected IoFailure, not an unsafe answer.
    let r = FakeResolver {
        answers: HashMap::new(),
    };
    let out = ScriptedTransport::new(vec![201]);
    let rng = FixedRng { seed: [4; 32] };
    let clock = ManualClock::new(1_700_000_000_000);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(pump.run_once(), Err(Error::Transport("dns-io")));
    assert!(no_ack(&p1));
    assert!(out.calls.borrow().is_empty());
    assert_eq!(HINT.len(), 32);
}

#[test]
fn invalid_eph_scalar_withholds_ack() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/push")]));
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![201]);
    // Zero scalar is an injected RNG failure, not a malformed recipient key.
    let rng = FixedRng { seed: [0; 32] };
    let clock = ManualClock::new(1_700_000_000_000);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(pump.run_once(), Err(Error::Transport("rng-eph")));
    assert!(no_ack(&p1));
    assert!(out.calls.borrow().is_empty());
}
