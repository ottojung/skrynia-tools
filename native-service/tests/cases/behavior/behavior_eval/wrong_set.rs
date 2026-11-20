use super::{AppPrecond, MemDisk, NoLock, Registry, auth};

#[test]
fn wrong_secret_cannot_set() {
    let disk = MemDisk::default();
    let lock = NoLock;
    let reg = Registry::open(&disk, "reg").unwrap();
    let admin = [5u8; 32];
    let _iid = reg.register(&disk, &lock, &admin, b"v1".to_vec()).unwrap();
    let bad_admin = [0u8; 32];
    let h1 = auth::hash_bytes(b"v1").to_vec();
    assert!(
        reg.set(
            &disk,
            &lock,
            &bad_admin,
            &AppPrecond::Present(h1),
            b"v2".to_vec()
        )
        .is_err()
    );
}
