use super::*;

#[test]
fn recursive_self_lookup() {
    let disk = Arc::new(MemDisk::default());
    let lock = Arc::new(NoLock);
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let admin = [1u8; 32];
    let id = reg
        .register(&*disk, &*lock, &admin, b"self".to_vec())
        .unwrap();
    let eval = SelfEval {
        reg: reg.clone(),
        disk: disk.clone(),
        lock: lock.clone(),
        me: id.clone(),
    };
    assert_eq!(
        reg.invoke(&*disk, &*lock, &eval, &id, b"abc").unwrap(),
        vec![0]
    );
}
