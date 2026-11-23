use crate::chain::{EdgeProc, Kernel};
use crate::provider::{FakePush, SubKeys};
use std::path::{Path, PathBuf};
use webpush_sender::p1::HttpP1;

pub struct Ctx {
    pub art: PathBuf,
    pub tmp: PathBuf,
    pub kernel: Kernel,
    pub edge: EdgeProc,
    pub io: HttpP1,
    pub push: FakePush,
    pub sub_keys: SubKeys,
    pub sub_json: Vec<u8>,
    pub sender_cap: [u8; 32],
    pub ns: [u8; 32],
    pub key: Vec<u8>,
    pub last_clock_sleeps: Vec<u64>,
    pub last_push_sent: usize,
}
impl Ctx {
    pub fn new(art: &Path, tmp: &Path) -> Self {
        let kernel = Kernel::start(art, &tmp.join("kernel")).expect("kernel");
        let edge =
            EdgeProc::start(art, &tmp.join("edge"), kernel.port, &kernel.token).expect("edge");
        let io = crate::chain::transport(&edge);
        let push = FakePush::start(crate::provider::ReplyPlan::status(201));
        let (sub_json, sub_keys) = super::super::keys::fixture();
        Self {
            art: art.into(),
            tmp: tmp.into(),
            kernel,
            edge,
            io,
            push,
            sub_keys,
            sub_json,
            sender_cap: [7u8; 32],
            ns: [9u8; 32],
            key: b"notice/alpha".to_vec(),
            last_clock_sleeps: Vec::new(),
            last_push_sent: 0,
        }
    }
}
