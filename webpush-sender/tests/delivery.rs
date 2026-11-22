mod common;

use common::{
    FakeP1, FixedRng, HINT, ManualClock, ScriptedTransport, config, frame, resolver, sub_json,
    vapid_key,
};
use webpush_sender::{Pump, Report, push::encrypt, sub::Subscription};

#[test]
fn delivers_opaque_hint_after_authoritative_reread() {
    let cfg = config();
    let mut p1 = FakeP1::new(vec![1], frame(&[sub_json("https://stale.example/push")]));
    let fresh = sub_json("https://fresh.example/push");
    p1.alternate_subs = Some(frame(std::slice::from_ref(&fresh)));
    let r = resolver(&[
        ("stale.example", "93.184.216.34"),
        ("fresh.example", "8.8.8.8"),
    ]);
    let out = ScriptedTransport::new(vec![201]);
    let rng = FixedRng { seed: [4u8; 32] };
    let clock = ManualClock::new(1_700_000_000_000);
    let key = vapid_key();
    let mut pump = Pump::new(&cfg, &p1, &out, &r, &key, &rng, &clock).unwrap();
    assert_eq!(
        pump.run_once().unwrap(),
        Report {
            hints: 1,
            delivered: 1,
            acked: 1,
            skipped: 0
        }
    );
    let calls = out.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].host, "fresh.example");
    assert_eq!(calls[0].ip.to_string(), "8.8.8.8");
    assert_eq!(calls[0].path, "/push");
    assert_eq!(calls[0].topic, [2u8; 16]);
    assert!(calls[0].authorization.starts_with("vapid t="));
    let expected = encrypt(&HINT, &Subscription::parse(&fresh).unwrap(), &rng).unwrap();
    assert_eq!(calls[0].body, expected);
    drop(calls);
    assert_eq!(p1.opcodes(), vec![20, 21, 23, 22]);
}
