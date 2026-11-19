use super::*;

#[test]
fn cyclic_dependency_lookup() {
    let disk = Arc::new(MemDisk::default());
    let lock = Arc::new(NoLock);
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let a = reg
        .register(&*disk, &*lock, &[2u8; 32], b"A".to_vec())
        .unwrap();
    let b = reg
        .register(&*disk, &*lock, &[3u8; 32], b"B".to_vec())
        .unwrap();
    let eval = CycleEval {
        reg: reg.clone(),
        disk: disk.clone(),
        lock: lock.clone(),
        a: a.clone(),
        b: b.clone(),
    };
    assert_eq!(
        reg.invoke(&*disk, &*lock, &eval, &a, b"xx").unwrap(),
        b"done"
    );
}
