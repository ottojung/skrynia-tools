use super::*;

#[test]
fn open_skips_corrupt_records() {
    let dir = tmp("corrupt");
    let disk = FsDisk(dir.clone());
    let lock = StdLock::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let admin = [13u8; 32];
    let id = reg.register(&disk, &lock, &admin, b"ok".to_vec()).unwrap();
    let active = dir.join("reg").join("active");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("not-a-record.txt"), b"junk").unwrap();
    fs::write(active.join("00.bin"), b"bad").unwrap();
    let reg2 = Registry::open(&disk, "reg").unwrap();
    assert_eq!(reg2.lookup(&disk, &id).unwrap(), b"ok");
    let _ = fs::remove_dir_all(dir);
}
