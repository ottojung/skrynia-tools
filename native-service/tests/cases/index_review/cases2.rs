//! Bounded removal and bounded retired COW nodes.
use super::support::{Memory, key};
use skrynia_native_service::index::{self, Scope};

#[test]
fn deleting_history_cannot_leave_unbounded_read_cost() {
    let d = Memory::default();
    let s = Scope { base: "reg" };
    for i in 1..=2500 {
        index::insert(&d, s, key(i)).unwrap();
    }
    for i in 1..2500 {
        index::remove(&d, s, key(i)).unwrap();
    }
    d.reads.set(0);
    assert_eq!(index::page(&d, s, &None, 256).unwrap(), vec![key(2500)]);
    assert!(
        d.reads.get() <= 32,
        "{} disk reads for a single live entry",
        d.reads.get()
    );
}
#[test]
fn retired_cow_generations_do_not_grow_without_bound() {
    let d = Memory::default();
    let s = Scope { base: "reg" };
    for _ in 0..100 {
        index::insert(&d, s, key(1)).unwrap();
    }
    assert!(
        d.files.borrow().len() <= 16,
        "{} files for one key",
        d.files.borrow().len()
    );
}
