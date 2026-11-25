//! Unsupported ready-marker format fails closed; explicit replay is migration.
use super::support::{setup, sorted};
use skrynia_native_service::{cap::Disk, store, store_io, store_io::StoreErr};

#[test]
fn unsupported_marker_fails_closed_then_owner_replay_migrates() {
    let (reg, d, lock, admin, iid, expected) = setup();
    let all = sorted(&expected);
    let target = all[0];
    let path = store::ready_path("reg", &iid, &target).unwrap();
    d.write_atomic(&path, &[2u8; 33]).unwrap();
    assert!(matches!(
        store_io::list_ready(&d, "reg", &iid),
        Err(StoreErr::BadRecord)
    ));
    assert!(reg.activate_release(&d, &lock, &admin, 1, &target).is_err());
    d.remove(&path).unwrap();
    let migrated = reg.stage_release(&d, &lock, &admin, b"v-migrated").unwrap();
    reg.seal_release(&d, &lock, &admin, &migrated).unwrap();
    let after = reg.list_ready(&d, &iid).unwrap();
    assert!(after.contains(&migrated));
    assert_eq!(after.len(), expected.len());
    let state = reg
        .activate_release(&d, &lock, &admin, 1, &migrated)
        .unwrap();
    assert_eq!(state.hash, migrated);
    assert_eq!(reg.lookup(&d, &iid).unwrap(), b"v-migrated");
}
