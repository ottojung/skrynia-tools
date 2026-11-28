//! Unsafe/malformed recipients stay per-item; only local failures are retried.
use super::{
    FakeP1, FakeResolver, FixedRng, ManualClock, ScriptedTransport, config, frame, resolver,
    sub_json, vapid_key,
};
use std::collections::HashMap;
use webpush_sender::{Error, Pump, sub::validate_url};

#[test]
fn unsafe_and_malformed_recipients_stay_per_item() {
    let cfg = config();
    for url in ["http://push.example/push", "https://127.0.0.1/push"] {
        let p1 = FakeP1::new(vec![1], frame(&[sub_json(url)]));
        let r = resolver(&[("push.example", "93.184.216.34")]);
        let out = ScriptedTransport::new(vec![201]);
        let rng = FixedRng { seed: [4; 32] };
        let clock = ManualClock::new(1_700_000_000_000);
        let key = vapid_key();
        let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
        assert!(pump.run_once().is_ok(), "{url} must stay per-item");
        assert!(out.calls.borrow().is_empty(), "{url} must never be sent");
        assert!(p1.opcodes().contains(&22), "{url} must not block ACK");
    }
    // A mixed DNS answer remains an SSRF verdict, not a retryable failure.
    let mut mixed = resolver(&[("push.example", "93.184.216.34")]);
    mixed
        .answers
        .get_mut("push.example")
        .unwrap()
        .push("127.0.0.1".parse().unwrap());
    assert_eq!(
        validate_url("https://push.example/p", &mixed),
        Err(Error::Ssrf("address"))
    );
    let empty = FakeResolver {
        answers: HashMap::new(),
    };
    assert_eq!(
        validate_url("https://push.example/p", &empty),
        Err(Error::Transport("dns-io"))
    );
    // Empty success is no usable answer: transient, not SSRF address denial.
    let mut no_answer = resolver(&[("push.example", "93.184.216.34")]);
    no_answer.answers.get_mut("push.example").unwrap().clear();
    assert_eq!(
        validate_url("https://push.example/p", &no_answer),
        Err(Error::Transport("dns-empty"))
    );
}
