//! Capacity exhaustion must not publish a pending program.
use super::legacy_support::Mem;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::index;
use skrynia_native_service::mlock::StdLock;
use skrynia_native_service::reg::Registry;

#[test]
fn exhausted_index_generation_does_not_publish_a_pending_program() {
    use sha2::{Digest, Sha256};
    let d = Mem::default();
    let l = StdLock::default();
    let r = Registry::open(&d, "reg").unwrap();
    r.register(&d, &l, &[5; 32], b"code".to_vec()).unwrap();
    let s = index::scope("reg");
    let mut m = d.read_all(&s.meta()).unwrap().unwrap();
    let old = u32::from_le_bytes(m[12..16].try_into().unwrap());
    let root = d.read_all(&s.root(old)).unwrap().unwrap();
    d.write_atomic(&s.root(u32::MAX), &root).unwrap();
    d.remove(&s.root(old)).unwrap();
    m.truncate(16);
    m[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    m.extend_from_slice(&Sha256::digest(&m));
    d.write_atomic(&s.meta(), &m).unwrap();
    let before = d.0.borrow().clone();
    assert!(r.register(&d, &l, &[6; 32], b"new".to_vec()).is_err());
    let after = d.0.borrow();
    assert_eq!(
        before
            .iter()
            .filter(|(p, _)| p.contains("/active/"))
            .collect::<Vec<_>>(),
        after
            .iter()
            .filter(|(p, _)| p.contains("/active/"))
            .collect::<Vec<_>>(),
        "capacity error changed visible active record"
    );
    assert!(
        index::take_pending(&d, s).unwrap().is_none(),
        "unrecoverable pending intent"
    );
}
