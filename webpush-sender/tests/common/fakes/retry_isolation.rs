//! One outstanding hint, same pump: DNS failure withholds ACK, recovery ACKs.
use super::{
    FakeP1, FixedRng, ManualClock, ScriptedResolver, ScriptedTransport, config, frame, sub_json,
    vapid_key,
};
use std::{cell::RefCell, collections::HashMap};
use webpush_sender::{Error, Pump};

#[test]
fn transient_dns_failure_withholds_ack_then_recovers_on_same_pump() {
    let cfg = config();
    // Single outstanding event; the healthy sub may be delivered twice.
    let p1 = FakeP1::new(
        vec![1],
        frame(&[
            sub_json("https://down.example/push"),
            sub_json("https://up.example/push"),
        ]),
    );
    // Initially only the healthy sub resolves; down.example is a transient
    // injected resolver I/O failure, not an unsafe answer.
    let mut answers = HashMap::new();
    answers.insert("up.example".into(), vec!["8.8.8.8".parse().unwrap()]);
    let r = ScriptedResolver {
        answers: RefCell::new(answers),
    };
    let out = ScriptedTransport::new(vec![201, 201, 201]);
    let rng = FixedRng { seed: [4; 32] };
    let clock = ManualClock::new(1_700_000_000_000);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();

    assert_eq!(pump.run_once(), Err(Error::Transport("dns-io")));
    assert!(!p1.opcodes().contains(&22), "no ACK before recovery");
    assert_eq!(out.calls.borrow().len(), 1, "healthy sub still sent");

    // Real recovery on the same resolver, same pump and same outstanding hint.
    r.set("down.example", "93.184.216.34");
    let report = pump.run_once().unwrap();
    assert!(out.calls.borrow().len() >= 2, "retry performed a real send");
    assert_eq!(report.acked, 1, "ACK only after recovery");
    assert_eq!(
        p1.opcodes().iter().filter(|&&o| o == 22).count(),
        1,
        "exactly one ACK"
    );
}
