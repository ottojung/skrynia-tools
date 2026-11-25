//! Legacy storage and exhausted-generation fail-closed regressions.
use super::legacy_support::Mem;
use skrynia_native_service::mlock::StdLock;
use skrynia_native_service::reg::Registry;

#[test]
fn marker_only_registry_must_not_silently_hide_existing_programs() {
    let d = Mem::default();
    let l = StdLock::default();
    let r = Registry::open(&d, "reg").unwrap();
    let original = r.register(&d, &l, &[5; 32], b"code".to_vec()).unwrap();
    d.0.borrow_mut().retain(|p, _| !p.contains("/index/"));
    let before = d.0.borrow().clone();
    let result = r.register(&d, &l, &[6; 32], b"new".to_vec());
    if result.is_ok() {
        let list = r.list_programs(&d, None, 256).unwrap();
        assert!(
            list.iter().any(|x| x.as_slice() == original.as_slice()),
            "previously active program disappeared from pagination"
        );
    } else {
        assert_eq!(
            *d.0.borrow(),
            before,
            "unsupported registry must reject before mutation"
        );
    }
}

#[test]
fn first_pending_replay_refuses_other_legacy_active_markers() {
    use skrynia_native_service::index;
    let d = Mem::default();
    let l = StdLock::default();
    let r = Registry::open(&d, "reg").unwrap();
    r.register(&d, &l, &[5; 32], b"first".to_vec()).unwrap();
    let second = r.register(&d, &l, &[6; 32], b"second".to_vec()).unwrap();
    d.0.borrow_mut().retain(|p, _| !p.contains("/index/"));
    index::write_pending(&d, index::scope("reg"), second.try_into().unwrap()).unwrap();
    let before = d.0.borrow().clone();
    assert!(r.list_programs(&d, None, 64).is_err());
    assert_eq!(*d.0.borrow(), before, "legacy refusal must precede replay");
}
