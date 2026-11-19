use super::*;

#[test]
fn durable_replace_and_recover() {
    let dir = tmp("dur");
    let disk = Arc::new(FsDisk(dir.clone()));
    let lock = StdLock::default();
    let reg = Registry::open(&*disk, "reg").unwrap();
    let admin = [11u8; 32];
    let id = reg.register(&*disk, &lock, &admin, b"v1".to_vec()).unwrap();
    let h1 = auth::hash_bytes(b"v1").to_vec();
    reg.set(
        &*disk,
        &lock,
        &admin,
        &AppPrecond::Present(h1),
        b"v2".to_vec(),
    )
    .unwrap();
    let reg2 = Registry::open(&*disk, "reg").unwrap();
    assert_eq!(reg2.lookup(&id).unwrap(), b"v2");
    assert_eq!(reg2.invoke(&*disk, &lock, &Tag, &id, b"").unwrap(), b"v2");
    let h2 = auth::hash_bytes(b"v2").to_vec();
    reg2.revoke(&*disk, &lock, &admin, &AppPrecond::Present(h2))
        .unwrap();
    let reg3 = Registry::open(&*disk, "reg").unwrap();
    assert!(reg3.lookup(&id).is_err());
    let _ = fs::remove_dir_all(dir);
}
