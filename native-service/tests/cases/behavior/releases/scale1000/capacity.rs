//! Honest capacity ceiling: bounded wire page, unbounded full-scan work.
use super::metered::Metered;
use super::support::{build, setup};
use crate::cases::behavior::releases::protocol_helpers::call;
use crate::cases::support::NoLock;
use skrynia_native_service::reg::Registry;
use skrynia_native_service::wire::{AdminRequest, Request, STATUS_ERR, STATUS_OK};

#[test]
fn capacity_ceiling_is_bounded_page_but_unbounded_scan() {
    let d = Metered::new();
    let lock = NoLock;
    let admin = vec![3u8; 32];
    let (iid, hashes) = build(&d, &lock, &admin);
    let reg = Registry::open(&d, "reg").unwrap();
    let before = d.ops();
    let all = reg.list_ready(&d, &iid).unwrap();
    let after = d.ops();
    assert_eq!(all.len(), hashes.len());
    assert_eq!(after.1 - before.1, 1, "one directory enumeration");
    assert!(
        after.0 - before.0 >= hashes.len(),
        "directory-scan listing reads every ready marker"
    );
    let (reg, d, lock, _a, id, _) = setup();
    let over = call(
        &reg,
        &d,
        &lock,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: id.clone(),
            after: None,
            limit: 257,
        }),
    );
    assert_eq!(over.status, STATUS_ERR);
    assert_eq!(over.body.first(), Some(&1));
    let ok = call(
        &reg,
        &d,
        &lock,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: id,
            after: None,
            limit: 256,
        }),
    );
    assert_eq!(ok.status, STATUS_OK);
}
