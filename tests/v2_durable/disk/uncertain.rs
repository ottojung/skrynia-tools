use super::{Fs, R, tmp};
use skrynia_tools::wire::v2compat::{DiskRandom, DiskStateStore, DurableBackend};
use std::{fs, sync::atomic::AtomicU64};

#[test]
fn persistent_uncertainty_fails_closed_on_open() {
    let dir = tmp("unc");
    let fs = Fs::new(dir.clone());
    let store = || DiskStateStore::new(fs.clone());
    let b = DurableBackend::open(store(), DiskRandom(R(AtomicU64::new(1)))).unwrap();
    b.bootstrap_root("r", "i").unwrap();
    drop(b);
    fs::create_dir_all(dir.join("v2durable")).unwrap();
    fs::write(dir.join("v2durable/uncertain"), b"1").unwrap();
    let err = DurableBackend::open(store(), DiskRandom(R(AtomicU64::new(2))))
        .err()
        .unwrap();
    assert!(err.contains("state_uncertain"));
    let _ = std::fs::remove_dir_all(dir);
}
