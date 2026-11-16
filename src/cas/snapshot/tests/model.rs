use super::{Commit, SnapshotStore, fake::Fake};
use std::collections::BTreeMap;

#[test]
fn long_state_machine_trace_preserves_atomic_snapshots() {
    let fake = Fake::default();
    let store = SnapshotStore::open(&fake, "model").unwrap();
    store
        .initialize("00000000000000000000000000000001")
        .unwrap();
    let mut oracle = BTreeMap::new();
    for n in 2..=258u64 {
        let old = store.read().unwrap();
        assert_eq!(old.values(), &oracle);
        let mut desired = oracle.clone();
        let key = format!("k{}", n.wrapping_mul(17) % 9);
        if n % 5 == 0 {
            desired.remove(&key);
        } else {
            desired.insert(key, n.to_be_bytes().to_vec());
        }
        *fake.uncertain_create.lock().unwrap() = n % 7 == 0;
        *fake.uncertain.lock().unwrap() = n % 11 == 0;
        *fake.fail_cas_before.lock().unwrap() = n % 13 == 0;
        let txid = format!("{n:032x}");
        let result = store.commit(&old, &txid, &desired);
        if n % 13 == 0 {
            assert!(result.is_err());
            assert_eq!(store.read().unwrap().values(), &oracle);
            assert_eq!(
                store.commit(&old, &txid, &desired).unwrap(),
                Commit::Committed
            );
        } else {
            assert_eq!(result.unwrap(), Commit::Committed);
        }
        oracle = desired;
        let current = store.read().unwrap();
        assert_eq!(current.revision(), n - 1);
        assert_eq!(current.values(), &oracle);
        if n % 23 == 0 {
            assert_eq!(
                store.commit(&old, &txid, &oracle).unwrap(),
                Commit::Committed
            );
        }
    }
}
