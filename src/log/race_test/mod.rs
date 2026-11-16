use super::{AppendLog, CreateResult, Store, tests::Fake};
use anyhow::Result;
use std::sync::{
    Barrier,
    atomic::{AtomicUsize, Ordering},
};
mod store;
use store::Coordinated;

#[test]
fn simultaneous_duplicate_operations_publish_one_slot() {
    let store = Coordinated::default();
    let log = AppendLog::open(&store, "same.id").unwrap();
    log.initialize().unwrap();
    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| log.append("00000000000000000000000000000001", b"same"));
        let b = scope.spawn(|| log.append("00000000000000000000000000000001", b"same"));
        (a.join().unwrap().unwrap(), b.join().unwrap().unwrap())
    });
    assert_eq!(results, (1, 1));
    assert_eq!(log.read_from(1, 3).unwrap().len(), 1);
}

#[test]
fn concurrent_id_collision_rejects_changed_payload() {
    let store = Coordinated::default();
    let log = AppendLog::open(&store, "different.payload").unwrap();
    log.initialize().unwrap();
    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| log.append("00000000000000000000000000000001", b"first"));
        let b = scope.spawn(|| log.append("00000000000000000000000000000001", b"second"));
        (a.join().unwrap(), b.join().unwrap())
    });
    assert!(results.0.is_ok() ^ results.1.is_ok());
    assert_eq!(log.read_from(1, 3).unwrap().len(), 1);
}
