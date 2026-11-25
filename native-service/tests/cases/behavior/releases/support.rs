use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::auth;
use skrynia_native_service::cap::Disk;
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

pub fn list_all<D: Disk>(reg: &Registry, d: &D, iid: &[u8]) -> Vec<[u8; 32]> {
    let mut out = Vec::new();
    let mut after = None;
    loop {
        let page = match reg.list_ready(d, iid, after, 256) {
            Ok(v) => v,
            Err(_) => panic!("list_ready failed"),
        };
        if page.is_empty() {
            break;
        }
        out.extend_from_slice(&page);
        if page.len() < 256 {
            break;
        }
        after = Some(*page.last().unwrap());
    }
    out
}
