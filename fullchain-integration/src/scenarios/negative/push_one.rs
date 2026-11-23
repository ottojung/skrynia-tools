//! One negative push-delivery attempt through the real chain.
use super::push_pump::{config, ledger, vapid_key};
use super::push_setup::{h, setup};
use crate::{
    EdgeProc, FakePush, FixedResolver, FixedRng, Kernel, LoopbackPush, ReplyPlan, TestClock,
    transport,
};
use std::path::Path;
use webpush_sender::pump::{Parts, Pump};

pub fn one(
    art: &Path,
    tmp: &Path,
    sub_json: &[u8],
    plan: ReplyPlan,
) -> (webpush_sender::Error, usize) {
    let k = Kernel::start(art, &tmp.join("k")).expect("k");
    let e = EdgeProc::start(art, &tmp.join("e"), k.port, &k.token).expect("e");
    let io = transport(&e);
    let admin = h(b"a");
    let sender = h(b"s");
    let key = b"k";
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    setup(&io, &echo, key, sub_json, &sender);
    let push = FakePush::start(plan);
    let cfg = config(sender, h(&admin), key);
    let res = FixedResolver::new("push.example.test", "93.184.216.34".parse().unwrap());
    let out = LoopbackPush::new(push.ca_der.clone(), push.addr.port());
    let vk = vapid_key();
    let (rng, clock) = (FixedRng, TestClock::new());
    let store = ledger(tmp);
    let mut p =
        Pump::with_ledger(Parts::new(&cfg, &io, &out, &res, &vk, &rng, &clock), store).unwrap();
    let err = p.run_once().unwrap_err();
    (err, push.seen().len())
}
