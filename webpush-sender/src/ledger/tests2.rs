//! Grant isolation and seq-reuse/migration invariants.
use super::model::test_cfg;
use super::{Decision, DurableLedger, Error, MemoryLedgerStore};

#[test]
fn foreign_grant_cannot_inherit_ledger() {
    let store = MemoryLedgerStore::new();
    let mut a = DurableLedger::open(store.clone(), &test_cfg()).unwrap();
    a.mark_delivered(1).unwrap();
    let mut other = test_cfg();
    other.sender_cap[0] ^= 1;
    assert_eq!(
        DurableLedger::open(store, &other).err(),
        Some(Error::Config("ledger-binding"))
    );
}

#[test]
fn reused_seq_fails_closed() {
    let mut l = DurableLedger::open(MemoryLedgerStore::new(), &test_cfg()).unwrap();
    l.mark_delivered(100).unwrap();
    assert_eq!(l.decide(5), Err(Error::Wire("ledger-reuse")));
}

#[test]
fn epoch_bump_cannot_suppress_with_stale_file() {
    let store = MemoryLedgerStore::new();
    let mut a = DurableLedger::open(store.clone(), &test_cfg()).unwrap();
    a.mark_delivered(100).unwrap();
    let mut next = test_cfg();
    next.ledger_epoch = 1;
    // A rebinding refuses the stale file: no cross-generation poison.
    assert_eq!(
        DurableLedger::open(store, &next).err(),
        Some(Error::Config("ledger-binding"))
    );
    // The new generation uses a new file and suppresses nothing from the old.
    let b = DurableLedger::open(MemoryLedgerStore::new(), &next).unwrap();
    assert_eq!(b.decide(100), Ok(Decision::Fresh));
}
