use super::fixture::{admin, server};
use crate::guest;
use skrynia_program0::{store, wire::AdminRequest};
#[test]
#[ignore = "requires independent kernel and operator-provisioned Program 0"]
fn unsealed_version_rejected_even_after_restart() {
    let (mut server, cap, iid) = server();
    let h = admin(
        &server.url,
        AdminRequest::StageCode {
            admin_cap: cap.clone(),
            code: guest::byte(90),
        },
    )
    .unwrap();
    server.restart().unwrap();
    assert!(
        admin(
            &server.url,
            AdminRequest::ActivateCode {
                admin_cap: cap.clone(),
                expected_revision: 1,
                target: h.clone(),
            }
        )
        .is_err()
    );
    admin(
        &server.url,
        AdminRequest::SealCode {
            admin_cap: cap.clone(),
            hash: h.clone(),
        },
    )
    .unwrap();
    let b = admin(
        &server.url,
        AdminRequest::ActivateCode {
            admin_cap: cap,
            expected_revision: 1,
            target: h,
        },
    )
    .unwrap();
    assert_eq!(store::decode_active(&b).unwrap().revision, 2);
    assert_eq!(
        admin(&server.url, AdminRequest::GetActive { invoke_id: iid }).unwrap(),
        b
    );
}
