//! Adversarial continuation: malformed, sparse, stale and terminal cursors.
use super::support::{setup, sorted};
use crate::cases::behavior::releases::protocol_helpers::{call, hashes};
use skrynia_native_service::wire::{AdminRequest, Request, STATUS_ERR, STATUS_OK};

#[test]
fn adversarial_cursors_never_skip_or_duplicate() {
    let (reg, d, lock, _a, iid, expected) = setup();
    let all = sorted(&expected);
    let bad = call(
        &reg,
        &d,
        &lock,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: iid.clone(),
            after: Some(vec![0; 31]),
            limit: 256,
        }),
    );
    assert_eq!(bad.status, STATUS_ERR);
    assert_eq!(bad.body.first(), Some(&1));
    let page = call(
        &reg,
        &d,
        &lock,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: iid.clone(),
            after: Some(all[500].to_vec()),
            limit: 256,
        }),
    );
    assert_eq!(page.status, STATUS_OK);
    assert_eq!(hashes(&page.body), all[501..757].to_vec());
    let tail = call(
        &reg,
        &d,
        &lock,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: iid,
            after: Some(all[1000].to_vec()),
            limit: 256,
        }),
    );
    assert_eq!(tail.status, STATUS_OK);
    assert!(hashes(&tail.body).is_empty());
}
