//! Ledger invariants and fail-closed fault injection with injected stores.
use super::model::test_cfg;
use super::{Decision, DurableLedger, Error, MemoryLedgerStore};

#[test]
fn bounded_ledger_never_silently_stops() {
    let mut l = DurableLedger::open(MemoryLedgerStore::new(), &test_cfg()).unwrap();
    for seq in 1..=1025 {
        l.mark_delivered(seq).unwrap();
    }
    assert_eq!(l.decide(1025), Ok(Decision::Delivered));
}

#[test]
fn restart_restores_ack_state() {
    let store = MemoryLedgerStore::new();
    let mut a = DurableLedger::open(store.clone(), &test_cfg()).unwrap();
    a.mark_delivered(1).unwrap();
    a.mark_acked(1).unwrap();
    let b = DurableLedger::open(store, &test_cfg()).unwrap();
    assert_eq!(b.decide(1), Ok(Decision::Acked));
}

#[test]
fn corruption_fails_closed() {
    let store = MemoryLedgerStore::new();
    store.set_bytes(Some(vec![0u8; 64]));
    assert!(DurableLedger::open(store, &test_cfg()).is_err());
}

#[test]
fn save_quota_never_advances_local_state() {
    let store = MemoryLedgerStore::new();
    let mut l = DurableLedger::open(store.clone(), &test_cfg()).unwrap();
    store.faults().borrow_mut().save = Some(Error::Quota);
    assert_eq!(l.mark_delivered(1), Err(Error::Quota));
    store.faults().borrow_mut().save = None;
    let again = DurableLedger::open(store, &test_cfg()).unwrap();
    assert_eq!(again.decide(1), Ok(Decision::Fresh));
}

#[test]
fn load_failure_fails_closed() {
    let store = MemoryLedgerStore::new();
    store.faults().borrow_mut().load = Some(Error::Transport("ledger-read"));
    assert_eq!(
        DurableLedger::open(store, &test_cfg()).err(),
        Some(Error::Transport("ledger-read"))
    );
}
