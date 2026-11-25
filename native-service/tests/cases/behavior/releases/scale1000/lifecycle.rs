//! Revoke and re-register keep activation single-pointer and nonce-bound.
use super::support::{build, sorted};
use crate::cases::support::{MemDisk, NoLock};
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wire::AppPrecond;

#[test]
fn revoke_and_reregister_preserves_activation_safety() {
    let d = MemDisk::default();
    let lock = NoLock;
    let admin = vec![8u8; 32];
    let (iid, hashes) = build(&d, &lock, &admin);
    let reg = Registry::open(&d, "reg").unwrap();
    let all = sorted(&hashes);
    let first = reg
        .activate_release(&d, &lock, &admin, 1, &all[10])
        .unwrap();
    reg.revoke(&d, &lock, &admin, &AppPrecond::Present(first.hash.to_vec()))
        .unwrap();
    let revoked = reg.active_info(&d, &iid).unwrap();
    assert!(!revoked.live);
    assert_eq!(revoked.revision, 3);
    assert!(
        reg.activate_release(&d, &lock, &admin, 3, &all[20])
            .is_err()
    );
    let again = reg.register(&d, &lock, &admin, b"v-base".to_vec()).unwrap();
    assert_eq!(again, iid);
    let fresh = reg.active_info(&d, &iid).unwrap();
    assert!(fresh.live);
    assert_eq!(fresh.revision, 4);
    assert!(
        reg.activate_release(&d, &lock, &admin, 1, &all[20])
            .is_err()
    );
    let ok = reg
        .activate_release(&d, &lock, &admin, 4, &all[20])
        .unwrap();
    assert_eq!(ok.revision, 5);
}
