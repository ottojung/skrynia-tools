use super::*;
#[test]
fn open_ignores_orphan_temp_and_foreign_names() {
    let dir = tmp("orph");
    let disk = FsDisk(dir.clone());
    let reg = Registry::open(&disk, "reg").unwrap();
    let id = reg.register(&disk, &Rng(21), b"ok".to_vec()).unwrap();
    let regdir = dir.join("reg");
    fs::create_dir_all(&regdir).unwrap();
    fs::write(regdir.join(".abc.tmp"), b"junk").unwrap();
    fs::write(regdir.join("notes.txt"), b"junk").unwrap();
    fs::write(regdir.join(format!("{}.tmp", "f".repeat(64))), b"j").unwrap();
    let reg2 = Registry::open(&disk, "reg").unwrap();
    assert_eq!(reg2.lookup(&id).unwrap(), b"ok");
    let ids = skrynia_program0::store_ls::list_ids(&disk, "reg").unwrap();
    assert_eq!(ids, vec![id]);
    let _ = fs::remove_dir_all(dir);
}
