//! Auth keeps fatal priority over later transient delivery failures.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedTransport, build, config, frame, resolver, sub_json,
    vapid_key,
};
use webpush_sender::{Error, vapid::VapidKey};

type Env = (webpush_sender::Config, VapidKey, FixedRng, ManualClock);

fn setup() -> Env {
    (
        config(),
        vapid_key(),
        FixedRng { seed: [4u8; 32] },
        ManualClock::new(0),
    )
}

/// A permanent auth rejection is not hidden by retry exhaustion in a later
/// subscription of the same hint; unrelated recipients are still attempted.
pub fn auth_not_masked_by_later_transient_sub() {
    let (cfg, key, rng, clock) = setup();
    let p1 = FakeP1::new(
        vec![1],
        frame(&[
            sub_json("https://a.example/p"),
            sub_json("https://b.example/p"),
        ]),
    );
    let r = resolver(&[
        ("a.example", "93.184.216.34"),
        ("b.example", "93.184.216.35"),
    ]);
    let out = ScriptedTransport::new(vec![401, 500, 500, 500]);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once(), Err(Error::Auth("push")));
    assert_eq!(out.calls.borrow().len(), 4);
    assert!(!p1.opcodes().contains(&22));
}

/// A transient ack error on an earlier seq must not mask a later permanent
/// auth rejection; fatal still wins in the accumulated disposition.
pub fn auth_outweighs_prior_ack_failure_across_seqs() {
    let (cfg, key, rng, clock) = setup();
    let p1 = FakeP1::new(vec![1, 2], frame(&[sub_json("https://a.example/p")]));
    p1.fail_ack.set(true);
    let r = resolver(&[("a.example", "93.184.216.34")]);
    let out = ScriptedTransport::new(vec![201, 401]);
    let mut pump = build(&cfg, &p1, &out, &r, &key, &rng, &clock);
    assert_eq!(pump.run_once(), Err(Error::Auth("push")));
    assert_eq!(out.calls.borrow().len(), 2);
}
