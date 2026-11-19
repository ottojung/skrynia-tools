use super::{fakes::R, faultstore::Faulty};
use skrynia_tools::wire::v2compat::{DurableBackend, StateStore, Store};
use std::sync::atomic::AtomicU64;
#[test]
fn failed_save_blocks_further_reads_and_writes() {
    let store = Faulty::default();
    let backend = DurableBackend::open(store.clone(), R(AtomicU64::new(1))).unwrap();
    store.set(1);
    assert_eq!(
        backend.bootstrap_root("r", "issuer").unwrap_err().code,
        "state_uncertain"
    );
    assert_eq!(
        backend.bootstrap_root("r", "issuer").unwrap_err().code,
        "state_uncertain"
    );
    assert_eq!(
        backend.read_object("key").err().unwrap().code,
        "state_uncertain"
    );
    assert!(store.load().unwrap().is_none());
    store.set(0);
    let recovered = DurableBackend::open(store, R(AtomicU64::new(3))).unwrap();
    recovered.bootstrap_root("r", "issuer").unwrap();
}
#[test]
fn lost_save_acknowledgement_requires_reload() {
    let store = Faulty::default();
    let backend = DurableBackend::open(store.clone(), R(AtomicU64::new(1))).unwrap();
    store.set(2);
    assert_eq!(
        backend.bootstrap_root("r", "issuer").unwrap_err().code,
        "state_uncertain"
    );
    assert_eq!(
        backend.read_object("key").err().unwrap().code,
        "state_uncertain"
    );
    store.set(0);
    let recovered = DurableBackend::open(store, R(AtomicU64::new(2))).unwrap();
    assert_eq!(
        recovered.bootstrap_root("r", "issuer").unwrap_err().code,
        "already_exists"
    );
    recovered.issue_user("r", "issuer").unwrap();
}
