use super::*;
#[test]
fn concurrent_set_keeps_valid_last_value() {
    let dir = tmp("cset");
    let disk = Arc::new(FsDisk(dir.clone()));
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(31), b"v0".to_vec()).unwrap();
    let mut hs = Vec::new();
    for t in 0u8..4 {
        let (r, d, pid) = (reg.clone(), disk.clone(), id.clone());
        hs.push(thread::spawn(move || {
            for i in 0u8..80 {
                let v = vec![b'a' + t, i];
                let _ = r.set(&*d, &pid, v);
            }
        }));
    }
    for h in hs {
        h.join().unwrap();
    }
    let v = reg.lookup(&id).unwrap();
    assert_eq!(v.len(), 2);
    assert!(matches!(v[0], b'a'..=b'd'));
    let v2 = Registry::open(&*disk, "reg").unwrap().lookup(&id).unwrap();
    assert_eq!(v, v2);
    assert_eq!(reg.invoke(&*disk, &Tag, &id, b"").unwrap(), v2);
    let _ = fs::remove_dir_all(dir);
}
