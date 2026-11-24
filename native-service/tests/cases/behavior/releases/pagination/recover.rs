use super::list_all;
use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::index::{self, Scope};
use skrynia_native_service::reg::Registry;
use skrynia_native_service::store;

//! A pending seal intent is committed or discarded on recovery.
#[test]
fn pending_seal_intent_is_recovered_or_discarded() {
    let d = MemDisk::default();
    let lock = NoLock;
    let reg = Registry::open(&d, "reg").unwrap();
    let admin = [3u8; 32];
    let iid = reg.register(&d, &lock, &admin, b"first".to_vec()).unwrap();
    let scope = Scope {
        base: &store::ready_dir("reg", &iid).unwrap(),
    };
    let h1 = reg.stage_release(&d, &lock, &admin, b"one").unwrap();
    let h2 = reg.stage_release(&d, &lock, &admin, b"two").unwrap();
    let marker = [&[1u8][..], &h1[..]].concat();
    index::write_pending(&d, scope, h1).unwrap();
    d.write_atomic(&store::ready_path("reg", &iid, &h1).unwrap(), &marker)
        .unwrap();
    assert!(list_all(&reg, &d, &iid).contains(&h1));
    assert!(index::take_pending(&d, scope).unwrap().is_none());
    index::write_pending(&d, scope, h2).unwrap();
    let listed = list_all(&reg, &d, &iid);
    assert!(!listed.contains(&h2));
    assert!(index::take_pending(&d, scope).unwrap().is_none());
}

