mod common;

use common::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};
use webpush_sender::{Error, Pump};

#[test]
fn private_endpoint_aborts_before_any_send() {
    let cfg = config();
    let p1 = FakeP1::new(vec![1], frame(&[sub_json("https://push.example/p")]));
    let r = resolver(&[("push.example", "10.1.2.3")]);
    let out = ScriptedTransport::new(vec![201]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert!(matches!(pump.run_once(), Err(Error::Ssrf("address"))));
    assert!(out.calls.borrow().is_empty());
    assert!(!p1.opcodes().contains(&22));
}

#[test]
fn invalid_config_and_frame_fail_closed() {
    let mut cfg = config();
    let p1 = FakeP1::new(vec![], vec![]);
    let r = resolver(&[("push.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(0);
    let key = vapid_key();
    assert!(Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).is_ok());
    cfg.key.clear();
    assert!(matches!(
        Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock),
        Err(Error::Config("key"))
    ));
    assert!(webpush_sender::sub::parse_frame(&[1, 0, 0, 0]).is_err());
}
