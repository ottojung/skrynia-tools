use super::{Commit, SnapshotStore, fake::Fake};
use std::collections::BTreeMap;

#[test]
fn uncertain_staging_is_reconciled_and_failed_cas_leaves_old_snapshot() {
    let fake = Fake::default();
    let store = SnapshotStore::open(&fake, "faults").unwrap();
    store
        .initialize("00000000000000000000000000000001")
        .unwrap();
    let old = store.read().unwrap();
    let desired = BTreeMap::from([("a".into(), b"hello".to_vec())]);
    *fake.uncertain_create.lock().unwrap() = true;
    *fake.fail_cas_before.lock().unwrap() = true;
    let error = store
        .commit(&old, "00000000000000000000000000000002", &desired)
        .unwrap_err();
    assert!(error.to_string().contains("outcome unknown"));
    let current = store.read().unwrap();
    assert_eq!(current.revision(), 0);
    assert!(current.values().is_empty());
    assert_eq!(
        store
            .commit(&old, "00000000000000000000000000000002", &desired)
            .unwrap(),
        Commit::Committed
    );
    assert_eq!(store.read().unwrap().values(), &desired);
}
