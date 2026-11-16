use super::{Commit, SnapshotStore, fake::Fake};
use std::collections::BTreeMap;
#[test]
fn detects_corruption_and_reconciles_lost_cas_response() {
    let fake = Fake::default();
    let store = SnapshotStore::open(&fake, "secure").unwrap();
    store
        .initialize("00000000000000000000000000000001")
        .unwrap();
    let base = store.read().unwrap();
    *fake.uncertain.lock().unwrap() = true;
    let data = BTreeMap::from([("doc".into(), b"v1".to_vec())]);
    assert_eq!(
        store
            .commit(&base, "00000000000000000000000000000002", &data)
            .unwrap(),
        Commit::Committed
    );
    assert_eq!(store.read().unwrap().values, data);
    let blob = store.read().unwrap().refs["doc"].key.clone();
    fake.items.lock().unwrap().insert(blob, b"damaged".to_vec());
    assert!(
        store
            .read()
            .unwrap_err()
            .to_string()
            .contains("hash mismatch")
    );
    assert!(store.commit(&base, "INVALID", &data).is_err());
}

#[test]
fn cannot_commit_snapshot_from_another_store_even_with_identical_head() {
    let fake = Fake::default();
    let a = SnapshotStore::open(&fake, "namespacea").unwrap();
    let b = SnapshotStore::open(&fake, "namespaceb").unwrap();
    let genesis = "00000000000000000000000000000001";
    a.initialize(genesis).unwrap();
    b.initialize(genesis).unwrap();
    let old = a.read().unwrap();
    let proposed = BTreeMap::from([("name".into(), b"wrong".to_vec())]);
    assert!(
        b.commit(&old, "00000000000000000000000000000002", &proposed)
            .is_err()
    );
    assert_eq!(b.read().unwrap().revision(), 0);
}
