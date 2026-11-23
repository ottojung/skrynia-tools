//! Separate disposable kernel and trusted-edge origin with local HTTPS push.
use crate::{EdgeProc, FakePush, Kernel, ReplyPlan, SubKeys, fixture, transport};
use std::path::{Path, PathBuf};
use webpush_sender::p1::HttpP1;
pub struct Ctx {
    _kernel: Kernel,
    _edge: EdgeProc,
    pub io: HttpP1,
    pub push: FakePush,
    pub sub_keys: SubKeys,
    pub sub_json: Vec<u8>,
    pub tmp: PathBuf,
}
impl Ctx {
    pub fn new(art: &Path, tmp: &Path) -> Self {
        let kernel = Kernel::start(art, &tmp.join("kernel")).expect("ordinary kernel");
        let edge = EdgeProc::start(art, &tmp.join("edge"), kernel.port, &kernel.token)
            .expect("ordinary edge");
        let io = transport(&edge);
        let push = FakePush::start(ReplyPlan::status(201));
        let (sub_json, sub_keys) = fixture();
        Self { _kernel: kernel, _edge: edge, io, push, sub_keys, sub_json,
            tmp: tmp.into() }
    }
}
