use super::*;

#[test]
fn durable_concurrent_invoke_during_replace() {
    let dir = tmp("durc");
    let disk = Arc::new(FsDisk(dir.clone()));
    let lock = Arc::new(StdLock::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let admin = [12u8; 32];
    let id = reg
        .register(&*disk, &*lock, &admin, b"v1".to_vec())
        .unwrap();
    let mut hs = Vec::new();
    for _ in 0..4 {
        let (r, d, l, pid) = (reg.clone(), disk.clone(), lock.clone(), id.clone());
        hs.push(thread::spawn(move || {
            for _ in 0..200 {
                let v = r.invoke(&*d, &*l, &Tag, &pid, b"").unwrap();
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
        Registry::open(&*disk, "reg")
            .unwrap()
            .lookup(&*disk, &id)
            .unwrap(),
        b"v2"
    );
    let _ = fs::remove_dir_all(dir);
}
