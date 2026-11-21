use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::auth;
use skrynia_native_service::reg::Registry;

pub fn fixture() -> (Registry, MemDisk, NoLock, Vec<u8>, Vec<u8>) {
    let d = MemDisk::default();
    let lock = NoLock;
    let reg = Registry::open(&d, "reg").unwrap();
    let admin = vec![9u8; 32];
    let id = reg.register(&d, &lock, &admin, b"first".to_vec()).unwrap();
    assert_eq!(id, auth::read_cap_vec(&admin));
    (reg, d, lock, admin, id)
}
