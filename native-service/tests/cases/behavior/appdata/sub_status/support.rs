//! Fixture: one registered object with an exact Subscribe grant.
use super::disk::Mem;
use skrynia_native_service::{NoLock, reg::Registry};

pub struct Fixture {
    pub disk: Mem,
    pub reg: Registry,
    pub grant: [u8; 32],
    pub key: Vec<u8>,
    pub subscriber: Vec<u8>,
    pub subscription: Vec<u8>,
}

pub fn setup() -> Fixture {
    let disk = Mem::default();
    let reg = Registry::open(&disk, "d").unwrap();
    let (admin, grant) = ([3u8; 32], [7u8; 32]);
    let key = b"apps/demo/notice".to_vec();
    let subscriber = b"subscriber-A".to_vec();
    let subscription = b"{\"endpoint\":\"https://example.test/push\"}".to_vec();
    let no_lock = NoLock;
    reg.register(&disk, &no_lock, &admin, b"code".to_vec())
        .unwrap();
    reg.push_grant(&disk, &no_lock, &admin, &grant, &key, &subscriber)
        .unwrap();
    reg.push_subscribe(&disk, &no_lock, &grant, &key, &subscriber, &subscription)
        .unwrap();
    Fixture {
        disk,
        reg,
        grant,
        key,
        subscriber,
        subscription,
    }
}
