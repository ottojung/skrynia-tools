//! Narrow residual regressions. RED at 460f276, GREEN after the scoped patch.
mod common;

use common::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, jwt_claims, one,
    sub_json, vapid_key,
};
use webpush_sender::Error;

#[test]
fn sub_claim_quote_cannot_inject_claims() {
    let evil = "mailto:x\",\"exp\":4102444800,\"x\":\"pwned";
    let jwt = vapid_key().sign("https://push.example", 1, evil).unwrap();
    let c = jwt_claims(&jwt);
    assert_eq!(c["exp"], 1);
    assert!(c.get("x").is_none());
}

#[test]
fn config_rejects_claim_injection_sub() {
    let mut cfg = config();
    cfg.vapid_sub = "mailto:x\",\"exp\":4102444800".to_owned();
    assert_eq!(cfg.validate(), Err(Error::Config("sub")));
    cfg.vapid_sub = "mailto:ops@example.invalid".to_owned();
    assert!(cfg.validate().is_ok());
}

#[test]
fn ttl_beyond_vapid_24h_is_rejected() {
    let mut cfg = config();
    cfg.ttl_secs = 86_401;
    assert_eq!(cfg.validate(), Err(Error::Config("ttl")));
    cfg.ttl_secs = 86_400;
    assert!(cfg.validate().is_ok());
}

fn topics_for(seqs: Vec<u64>) -> Vec<[u8; 16]> {
    let cfg = config();
    let p1 = FakeP1::new(seqs, frame(&[sub_json("https://push.example/p")]));
    let r = one("93.184.216.34");
    let out = ScriptedTransport::new(vec![201; 4]);
    let (rng, clock) = (FixedRng { seed: [4u8; 32] }, ManualClock::new(0));
    let key = vapid_key();
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    pump.run_once().unwrap();
    out.calls.borrow().iter().map(|q| q.topic).collect()
}

#[test]
fn distinct_hints_use_distinct_topics() {
    let t = topics_for(vec![1, 2]);
    assert_ne!(t[0], t[1]);
}

#[test]
fn replayed_hint_keeps_topic() {
    assert_eq!(topics_for(vec![7])[0], topics_for(vec![7])[0]);
}
