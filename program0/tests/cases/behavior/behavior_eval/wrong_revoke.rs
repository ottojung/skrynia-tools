use super::{AppPrecond, GuardEval, MemDisk, NoLock, Registry, auth};

#[test]
fn wrong_secret_cannot_revoke_or_lookup() {
    let disk = MemDisk::default();
    let lock = NoLock;
    let reg = Registry::open(&disk, "reg").unwrap();
    let admin = [6u8; 32];
    let _iid = reg.register(&disk, &lock, &admin, b"v1".to_vec()).unwrap();
    let bad_admin = [0u8; 32];
    let bad_iid = auth::read_cap_vec(&bad_admin);
    let h1 = auth::hash_bytes(b"v1").to_vec();
    assert!(
        reg.revoke(&disk, &lock, &bad_admin, &AppPrecond::Present(h1))
            .is_err()
    );
    assert!(reg.lookup(&bad_iid).is_err());
    assert!(reg.invoke(&disk, &lock, &GuardEval, &bad_iid, b"").is_err());
}
