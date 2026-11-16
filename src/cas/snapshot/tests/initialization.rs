use super::{SnapshotStore, fake::Fake};
#[test]
fn simultaneous_initializers_end_with_one_valid_root() {
    let fake = Fake::default();
    let store = SnapshotStore::open(&fake, "bootstrap").unwrap();
    std::thread::scope(|scope| {
        for n in 1..=8u64 {
            let store = &store;
            scope.spawn(move || store.initialize(&format!("{n:032x}")).unwrap());
        }
    });
    assert_eq!(store.read().unwrap().revision(), 0);
    assert!(store.read().unwrap().values().is_empty());
}
