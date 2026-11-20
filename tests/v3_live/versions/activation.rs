use super::fixture::{admin, ready, server};
use crate::{guest, wire};
use skrynia_native_service::{auth, store, wire::AdminRequest};
#[test]
#[ignore = "requires independent kernel and operator-provisioned Program 0"]
fn native_release_activation_and_rollback_survive_restart() {
    let (mut server, cap, iid) = server();
    let initial = auth::hash_bytes(&guest::byte(65));
    let h2 = ready(&server.url, &cap, guest::byte(66));
    let changed = admin(
        &server.url,
        AdminRequest::ActivateCode {
            admin_cap: cap.clone(),
            expected_revision: 1,
            target: h2.clone(),
        },
    )
    .unwrap();
    assert_eq!(store::decode_active(&changed).unwrap().revision, 2);
    assert_eq!(wire::invoke(&server.url, &iid).unwrap(), b"B");
    server.restart().unwrap();
    assert_eq!(wire::invoke(&server.url, &iid).unwrap(), b"B");
    let restored = admin(
        &server.url,
        AdminRequest::ActivateCode {
            admin_cap: cap.clone(),
            expected_revision: 2,
            target: initial.to_vec(),
        },
    )
    .unwrap();
    assert_eq!(store::decode_active(&restored).unwrap().revision, 3);
    assert_eq!(wire::invoke(&server.url, &iid).unwrap(), b"A");
    assert!(
        admin(
            &server.url,
            AdminRequest::ActivateCode {
                admin_cap: cap,
                expected_revision: 1,
                target: h2,
            }
        )
        .is_err(),
        "stale A->B->A version must not be accepted"
    );
}
