use super::{Fs, R, tmp};
use skrynia_tools::wire::v2compat::{
    DiskRandom, DiskStateStore, DurableBackend, Mutation, MutationKind, ObjectRead, Store,
};
use std::sync::atomic::AtomicU64;

#[test]
fn restart_persists_on_program0_disk() {
    let dir = tmp("disk");
    let fs = Fs::new(dir.clone());
    let store = || DiskStateStore::new(fs.clone());
    let b = DurableBackend::open(store(), DiskRandom(R(AtomicU64::new(1)))).unwrap();
    b.bootstrap_root("r", "i").unwrap();
    let rt = b.issue_user("r", "i").unwrap()["userToken"]
        .as_str()
        .unwrap()
        .to_string();
    b.mutate_object(Mutation {
        key: "a",
        kind: MutationKind::Create,
        token: &rt,
        expected: None,
        body: b"hi",
    })
    .unwrap();
    drop(b);
    let b2 = DurableBackend::open(store(), DiskRandom(R(AtomicU64::new(9)))).unwrap();
    let ObjectRead::Found(obj) = b2.read_object("a").unwrap() else {
        panic!("missing");
    };
    assert_eq!(obj.bytes, b"hi");
    let _ = std::fs::remove_dir_all(dir);
}
