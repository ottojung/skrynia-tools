//! Restart/recovery and final-marker-removal fault injection at scale.
use super::support::{setup, sorted};
use skrynia_native_service::{cap::Disk, reg::Registry, store};

#[test]
fn restart_preserves_pointer_and_unsealed_stage_stays_inactive() {
    let (reg, d, lock, admin, iid, expected) = setup();
    let all = sorted(&expected);
    let staged = reg
        .stage_release(&d, &lock, &admin, b"uncommitted")
        .unwrap();
    reg.activate_release(&d, &lock, &admin, 1, &all[3]).unwrap();
    drop(reg);
    let reg = Registry::open(&d, "reg").unwrap();
    let state = reg.active_info(&d, &iid).unwrap();
    assert!(state.live);
    assert_eq!(state.revision, 2);
    assert_eq!(state.hash, all[3]);
    assert_eq!(reg.list_ready(&d, &iid).unwrap(), all);
    assert!(reg.activate_release(&d, &lock, &admin, 2, &staged).is_err());
    let next = reg.activate_release(&d, &lock, &admin, 2, &all[4]).unwrap();
    assert_eq!(next.revision, 3);
}

#[test]
fn final_removal_of_ready_marker_fails_closed_until_replay() {
    let (reg, d, lock, admin, iid, expected) = setup();
    let all = sorted(&expected);
    let target = all[7];
    let path = store::ready_path("reg", &iid, &target).unwrap();
    d.remove(&path).unwrap();
    let after = reg.list_ready(&d, &iid).unwrap();
    assert_eq!(after.len(), expected.len() - 1);
    assert!(!after.contains(&target));
    assert!(reg.activate_release(&d, &lock, &admin, 1, &target).is_err());
    reg.seal_release(&d, &lock, &admin, &target).unwrap();
    assert_eq!(reg.list_ready(&d, &iid).unwrap(), all);
    let ok = reg.activate_release(&d, &lock, &admin, 1, &target).unwrap();
    assert_eq!(ok.hash, target);
}
