use super::*;
use skrynia_native_service::wire::AdminRequest;

/// A staged-but-unsealed hash and a never-staged forged hash must both be
/// rejected by activation, leaving the revisioned active pointer untouched.
/// This is the source-level companion to e2e/real_http/rollback_e2e.py.
#[test]
fn unready_and_forged_hashes_cannot_be_activated() {
    let (r, d, l, a, id) = fixture();
    let first = r.active_info(&d, &id).unwrap();
    let staged = r.stage_release(&d, &l, &a, b"unready").unwrap();
    let res = call(
        &r,
        &d,
        &l,
        Request::Admin(AdminRequest::ActivateCode {
            admin_cap: a.clone(),
            expected_revision: first.revision,
            target: staged.to_vec(),
        }),
    );
    assert_eq!(res.status, STATUS_ERR);
    assert_eq!(r.active_info(&d, &id).unwrap().hash, first.hash);

    let forged = [0xA5u8; 32];
    let res = call(
        &r,
        &d,
        &l,
        Request::Admin(AdminRequest::ActivateCode {
            admin_cap: a.clone(),
            expected_revision: first.revision,
            target: forged.to_vec(),
        }),
    );
    assert_eq!(res.status, STATUS_ERR);
    assert_eq!(r.active_info(&d, &id).unwrap().revision, first.revision);

    r.seal_release(&d, &l, &a, &staged).unwrap();
    let res = call(
        &r,
        &d,
        &l,
        Request::Admin(AdminRequest::ActivateCode {
            admin_cap: a,
            expected_revision: first.revision,
            target: staged.to_vec(),
        }),
    );
    assert_eq!(res.status, STATUS_OK);
    assert_eq!(
        store::decode_active(&res.body).unwrap().revision,
        first.revision + 1
    );
}
