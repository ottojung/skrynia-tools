use super::fakes::{M, R};
use skrynia_tools::wire::v2compat::DurableBackend;
use std::sync::atomic::AtomicU64;

pub fn go() {
    let s = M::default();
    let b = DurableBackend::open(s.clone(), R(AtomicU64::new(1))).unwrap();
    b.bootstrap_root("r", "i").unwrap();
    let rt = b.issue_user("r", "i").unwrap()["userToken"]
        .as_str()
        .unwrap()
        .to_string();
    let ctx = super::setup::create_and_mutate(&b, &rt);
    drop(b);
    let b = DurableBackend::open(s, R(AtomicU64::new(9))).unwrap();
    super::verify::check(&b, &ctx);
}
