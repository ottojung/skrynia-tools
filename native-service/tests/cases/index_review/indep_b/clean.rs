//! A clean registry lists empty without creating files.
use super::legacy_support::Mem;
use skrynia_native_service::reg::Registry;

#[test]
fn clean_registry_lists_empty_without_mutating() {
    let d = Mem::default();
    let r = Registry::open(&d, "reg").unwrap();
    assert!(r.list_programs(&d, None, 64).unwrap().is_empty());
    assert!(d.0.borrow().is_empty());
}
