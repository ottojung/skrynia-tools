use super::{Commit, SnapshotStore, fake::Fake};
use std::collections::BTreeMap;
#[test]
fn concurrent_writers_never_both_commit_from_same_head() {
    let fake = Fake::default();
    let store = SnapshotStore::open(&fake, "conflict").unwrap();
    store
        .initialize("00000000000000000000000000000001")
        .unwrap();
    let old = store.read().unwrap();
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for n in 2..=9u64 {
            let store = &store;
            let old = &old;
            handles.push(scope.spawn(move || {
                let desired = BTreeMap::from([("value".into(), vec![n as u8])]);
                store.commit(old, &format!("{n:032x}"), &desired).unwrap()
            }));
        }
        handles
            .into_iter()
            .map(|job| job.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        results.iter().filter(|r| **r == Commit::Committed).count(),
        1
    );
    assert_eq!(store.read().unwrap().revision, 1);
}
