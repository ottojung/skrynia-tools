//! The program-ID index lists many IDs and tombstones revoked ones.
use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::reg::Registry;
use skrynia_native_service::{store, wire::AppPrecond};

fn all(reg: &Registry, d: &MemDisk, limit: u16) -> Vec<[u8; 32]> {
    let (mut out, mut after) = (Vec::new(), None);
    loop {
        let p = reg.list_programs(d, after, limit).unwrap();
        if p.len() < limit as usize {
            out.extend_from_slice(&p);
            return out;
        }
        after = Some(*p.last().unwrap());
        out.extend_from_slice(&p);
    }
}

#[test]
fn program_index_lists_many_ids_and_tombstones_revoked() {
    let (d, lock) = (MemDisk::default(), NoLock);
    let reg = Registry::open(&d, "reg").unwrap();
    let mut ids = Vec::new();
    for i in 0..250u8 {
        let mut cap = [0u8; 32];
        cap[0] = i;
        cap[1] = 1;
        let id: [u8; 32] = reg
            .register(&d, &lock, &cap, format!("app-{i}").into_bytes())
            .unwrap()
            .try_into()
            .unwrap();
        ids.push(id);
    }
    ids.sort();
    assert_eq!(all(&reg, &d, 64), ids);
    let mut cap = [0u8; 32];
    cap[0] = 7;
    cap[1] = 1;
    let id7 = store::invoke_id(&cap);
    let state = reg.active_info(&d, &id7).unwrap();
    reg.revoke(&d, &lock, &cap, &AppPrecond::Present(state.hash.to_vec()))
        .unwrap();
    let got = all(&reg, &d, 256);
    assert!(!got.contains(&id7));
    assert_eq!(got.len(), 249);
}
