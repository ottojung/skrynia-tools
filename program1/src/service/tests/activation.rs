use super::support::{Mem, err, ok, open_p};
use skrynia_native_service::wire::{AdminRequest, Request};
use skrynia_native_service::{auth, store};

#[test]
fn activation_rollback_and_aba_revision() {
    let d = Mem::default();
    let reg = open_p(&d);
    let admin = vec![7u8; 32];
    ok(
        &reg,
        &d,
        Request::Register {
            operator_cap: vec![85; 32],
            admin_cap: admin.clone(),
            code: b"a".to_vec(),
        },
    );
    let h2 = ok(
        &reg,
        &d,
        Request::Admin(AdminRequest::StageCode {
            admin_cap: admin.clone(),
            code: b"b".to_vec(),
        }),
    );
    ok(
        &reg,
        &d,
        Request::Admin(AdminRequest::SealCode {
            admin_cap: admin.clone(),
            hash: h2.clone(),
        }),
    );
    let a = ok(
        &reg,
        &d,
        Request::Admin(AdminRequest::ActivateCode {
            admin_cap: admin.clone(),
            expected_revision: 1,
            target: h2.clone(),
        }),
    );
    assert_eq!(store::decode_active(&a).unwrap().revision, 2);
    let h1 = auth::hash_bytes(b"a").to_vec();
    let b = ok(
        &reg,
        &d,
        Request::Admin(AdminRequest::ActivateCode {
            admin_cap: admin.clone(),
            expected_revision: 2,
            target: h1,
        }),
    );
    assert_eq!(store::decode_active(&b).unwrap().revision, 3);
    let _ = err(
        &reg,
        &d,
        Request::Admin(AdminRequest::ActivateCode {
            admin_cap: admin,
            expected_revision: 1,
            target: h2,
        }),
    );
}
