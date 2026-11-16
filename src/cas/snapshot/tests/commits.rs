use super::{Commit, SnapshotStore, fake::Fake};
use std::collections::BTreeMap;
#[test]
fn multiple_entries_are_published_at_one_cas_point() {
    let fake = Fake::default();
    let store = SnapshotStore::open(&fake, "documents").unwrap();
    store
        .initialize("00000000000000000000000000000001")
        .unwrap();
    let base = store.read().unwrap();
    assert_eq!(base.revision, 0);
    let mut values = BTreeMap::new();
    values.insert("a".into(), b"first".to_vec());
    values.insert("b".into(), b"second".to_vec());
    assert_eq!(
        store
            .commit(&base, "00000000000000000000000000000002", &values)
            .unwrap(),
        Commit::Committed
    );
    assert_eq!(
        store
            .commit(&base, "00000000000000000000000000000002", &values)
            .unwrap(),
        Commit::Committed
    );
    let version1 = store.read().unwrap();
    assert_eq!(version1.values, values);
    values.insert("b".into(), b"updated".to_vec());
    assert_eq!(
        store
            .commit(&version1, "00000000000000000000000000000003", &values)
            .unwrap(),
        Commit::Committed
    );
    let version2 = store.read().unwrap();
    assert_eq!(version2.revision, 2);
    assert_eq!(version2.values, values);
    assert_eq!(version1.refs["a"], version2.refs["a"]);
    assert_ne!(version1.refs["b"], version2.refs["b"]);
}
