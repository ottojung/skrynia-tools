use super::super::p1::must;
use super::super::wire::{blb, idb, req};
use crate::caps::{FixedResolver, FixedRng, TestClock};
use crate::chain::{EdgeProc, Kernel, op_cap, transport};
use crate::provider::{FakePush, ReplyPlan};
use crate::scenarios::transport::LoopbackPush;
use sha2::{Digest, Sha256};
use webpush_sender::ledger::FileLedgerStore;
use webpush_sender::pump::{Parts, Pump};
use webpush_sender::{Config, vapid::VapidKey};
fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let (sub_json, _) = crate::scenarios::keys::fixture();
    let mut plan302 = ReplyPlan::status(302);
    plan302
        .headers
        .push(("Location".into(), "https://evil.test/x".into()));
    let p302 = one(art, &tmp.join("r302"), &sub_json, plan302);
    let p503 = one(
        art,
        &tmp.join("r503"),
        &sub_json,
        ReplyPlan::status(503).retry(1),
    );
    vec![
        format!("neg_redirect_err={p302}"),
        format!("neg_backoff_attempts={p503}"),
    ]
}
fn one(art: &std::path::Path, tmp: &std::path::Path, sub_json: &[u8], plan: ReplyPlan) -> String {
    let k = Kernel::start(art, &tmp.join("k")).expect("k");
    let e = EdgeProc::start(art, &tmp.join("e"), k.port, &k.token).expect("e");
    let io = transport(&e);
    let admin = h(b"a");
    let grant = h(b"g");
    let sender = h(b"s");
    let key = b"k";
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    must(
        &io,
        &req(15, &[&idb(&op_cap()), &idb(&admin), &blb(&echo)]),
        "r",
    );
    must(
        &io,
        &req(18, &[&idb(&admin), &idb(&grant), &blb(key), &blb(b"sub")]),
        "g",
    );
    must(
        &io,
        &req(16, &[&idb(&grant), &blb(key), &blb(b"sub"), &blb(sub_json)]),
        "s",
    );
    must(
        &io,
        &req(6, &[&idb(&admin), &blb(key), &[0u8], &blb(b"v")]),
        "p",
    );
    must(
        &io,
        &req(19, &[&idb(&admin), &idb(&sender), &blb(key)]),
        "sg",
    );
    let push = FakePush::start(plan);
    let cfg = Config {
        sender_cap: sender,
        ns: h(&admin),
        key: key.to_vec(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2u8; 16],
        vapid_sub: "mailto:ops@example.invalid".into(),
        ledger_epoch: 0,
    };
    let res = FixedResolver::new("push.example.test", "93.184.216.34".parse().unwrap());
    let out = LoopbackPush::new(push.ca_der.clone(), push.addr.port());
    let vk = VapidKey::from_slice(&[3u8; 32]).unwrap();
    let (rng, clock) = (FixedRng, TestClock::new());
    std::fs::create_dir_all(tmp.join("l")).unwrap();
    let mut b = [0u8; 32];
    b.copy_from_slice(&Sha256::digest(b"b"));
    let store = FileLedgerStore::new(&tmp.join("l"), &b).unwrap();
    let mut p =
        Pump::with_ledger(Parts::new(&cfg, &io, &out, &res, &vk, &rng, &clock), store).unwrap();
    let err = p.run_once().unwrap_err();
    format!("{err:?}/seen={}", push.seen().len())
}
