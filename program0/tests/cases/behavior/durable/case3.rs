use super::*;
#[test]
fn randomized_restart_set_revoke_consistency() {
    let dir = tmp("rand");
    let disk = FsDisk(dir.clone());
    let mut reg = Registry::open(&disk, "reg").unwrap();
    let mut ids = Vec::new();
    let mut live: std::collections::BTreeMap<Vec<u8>, Vec<u8>> = std::collections::BTreeMap::new();
    let mut x: u64 = 0xC0FFEE;
    let mut next = || {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        x
    };
    for step in 0u8..120 {
        let op = (next() % 4) as u8;
        if op == 0 || ids.is_empty() {
            let id = reg.register(&disk, &Rng(step), vec![step; 3]).unwrap();
            live.insert(id.clone(), vec![step; 3]);
            ids.push(id);
        } else if op == 1 {
            let i = (next() as usize) % ids.len();
            let id = ids[i].clone();
            let v = vec![step; 4];
            if reg.set(&disk, &id, v.clone()).is_ok() {
                live.insert(id, v);
            }
        } else if op == 2 {
            let i = (next() as usize) % ids.len();
            let id = ids.remove(i);
            if reg.revoke(&disk, &id).is_ok() {
                live.remove(&id);
            }
        }
        if step % 15 == 14 {
            reg = Registry::open(&disk, "reg").unwrap();
        }
        let reg2 = Registry::open(&disk, "reg").unwrap();
        for (id, v) in &live {
            assert_eq!(reg2.lookup(id).as_ref().ok(), Some(v));
        }
    }
    let _ = fs::remove_dir_all(dir);
}
