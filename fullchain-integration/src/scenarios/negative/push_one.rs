//! Runs one negative push scenario against a fresh full chain.
use super::push_cfg::{config, drive};
use super::push_seed::install;
use crate::chain::{EdgeProc, Kernel, transport};
use crate::provider::{FakePush, ReplyPlan};
use std::path::Path;

pub fn one(art: &Path, tmp: &Path, sub_json: &[u8], plan: ReplyPlan) -> String {
    let k = Kernel::start(art, &tmp.join("k")).expect("k");
    let e = EdgeProc::start(art, &tmp.join("e"), k.port, &k.token).expect("e");
    let io = transport(&e);
    let (admin, sender) = install(&io, art, sub_json);
    let push = FakePush::start(plan);
    let cfg = config(&admin, &sender, b"k");
    drive(&cfg, &io, &push, tmp)
}
