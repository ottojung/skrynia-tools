use super::{GuardEval, MemDisk, NoLock, Registry};

#[test]
fn unsafe_import_rejected() {
    let disk = MemDisk::default();
    let lock = NoLock;
    let reg = Registry::open(&disk, "reg").unwrap();
    let admin = [4u8; 32];
    let iid = reg
        .register(&disk, &lock, &admin, b"UNSAFE_IMPORT".to_vec())
        .unwrap();
    assert!(reg.invoke(&disk, &lock, &GuardEval, &iid, b"").is_err());
}
