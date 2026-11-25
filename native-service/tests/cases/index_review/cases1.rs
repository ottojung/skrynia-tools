//! Zero key and fail-closed committed root.
use super::support::{Memory, key};
use skrynia_native_service::cap::Disk;
use skrynia_native_service::index::{self, Scope};

#[test]
fn zero_key_survives_second_insert() {
    let d = Memory::default();
    let s = Scope { base: "reg" };
    index::insert(&d, s, key(0)).unwrap();
    index::insert(&d, s, key(1)).unwrap();
    assert_eq!(
        index::page(&d, s, &None, 256).unwrap(),
        vec![key(0), key(1)]
    );
}
#[test]
fn missing_committed_root_is_an_error() {
    let d = Memory::default();
    let s = Scope { base: "reg" };
    index::insert(&d, s, key(1)).unwrap();
    d.remove(&s.root(1)).unwrap();
    assert!(index::page(&d, s, &None, 256).is_err());
    assert!(index::insert(&d, s, key(2)).is_err());
}
