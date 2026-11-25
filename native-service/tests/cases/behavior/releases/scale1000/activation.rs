//! O(1) single-pointer activation with a full 1001-version history resident.
use super::metered::Metered;
use super::support::{N, build, sorted};
use crate::cases::support::NoLock;
use skrynia_native_service::reg::Registry;

#[test]
fn activation_is_constant_work_across_arbitrary_history() {
    let d = Metered::new();
    let lock = NoLock;
    let admin = vec![7u8; 32];
    let (iid, hashes) = build(&d, &lock, &admin);
    let reg = Registry::open(&d, "reg").unwrap();
    let all = sorted(&hashes);
    let before = d.ops();
    let a = reg.activate_release(&d, &lock, &admin, 1, &all[0]).unwrap();
    let mid = d.ops();
    assert_eq!(a.revision, 2);
    assert!(mid.0 - before.0 <= 2, "pointer + direct marker reads only");
    assert_eq!(mid.1 - before.1, 0, "no history enumeration");
    assert!(mid.2 - before.2 <= 42, "one fixed pointer write");
    let b = reg.activate_release(&d, &lock, &admin, 2, &all[N]).unwrap();
    assert_eq!(b.revision, 3);
    assert_eq!(b.hash, all[N]);
    let c = reg.activate_release(&d, &lock, &admin, 3, &all[0]).unwrap();
    assert_eq!(c.revision, 4);
    assert_eq!(
        reg.lookup(&d, &iid).unwrap(),
        skrynia_native_service::store_io::read_blob(&d, "reg", &all[0]).unwrap()
    );
    assert!(reg.activate_release(&d, &lock, &admin, 2, &all[N]).is_err());
}
