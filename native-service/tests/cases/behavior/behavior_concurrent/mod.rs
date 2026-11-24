mod eval;
use crate::cases::support::{MemDisk, StdLock};
use eval::TagEval;
use skrynia_native_service::auth;
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wire::AppPrecond;
use std::sync::Arc;
use std::thread;

#[test]
fn concurrent_invoke_sees_v1_or_v2() {
    let disk = Arc::new(MemDisk::default());
    let lock = Arc::new(StdLock::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let admin = [3u8; 32];
    let id = reg
        .register(&*disk, &*lock, &admin, b"v1".to_vec())
        .unwrap();
    let mut hs = Vec::new();
    for _ in 0..4 {
        let (r, d, l, pid) = (reg.clone(), disk.clone(), lock.clone(), id.clone());
        hs.push(thread::spawn(move || {
            for _ in 0..200 {
                let v = r.invoke(&*d, &*l, &TagEval, &pid, b"").unwrap();
                assert!(v == b"v1" || v == b"v2");
            }
        }));
    }
    let h1 = auth::hash_bytes(b"v1").to_vec();
    reg.set(
        &*disk,
        &*lock,
        &admin,
        &AppPrecond::Present(h1),
        b"v2".to_vec(),
    )
    .unwrap();
    for h in hs {
        h.join().unwrap();
    }
    assert_eq!(
        reg.invoke(&*disk, &*lock, &TagEval, &id, b"").unwrap(),
        b"v2"
    );
}
