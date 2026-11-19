use super::fixture::{admin, ready, server};
use crate::guest;
use skrynia_program0::wire::AdminRequest;
#[test]
#[ignore = "requires independent kernel and operator-provisioned Program 0"]
fn native_history_pages_are_public_and_deterministic() {
    let (server, cap, iid) = server();
    let h = ready(&server.url, &cap, guest::byte(90));
    let all = admin(
        &server.url,
        AdminRequest::ListVersions {
            invoke_id: iid.clone(),
            after: None,
            limit: 256,
        },
    )
    .unwrap();
    assert_eq!(u16::from_le_bytes(all[..2].try_into().unwrap()), 2);
    assert_eq!(all.len(), 66);
    assert!(all[2..].chunks_exact(32).any(|x| x == h));
    let first = all[2..34].to_vec();
    let later = admin(
        &server.url,
        AdminRequest::ListVersions {
            invoke_id: iid.clone(),
            after: Some(first),
            limit: 1,
        },
    )
    .unwrap();
    assert_eq!(later.len(), 34);
    assert_eq!(u16::from_le_bytes(later[..2].try_into().unwrap()), 1);
    let programs = admin(
        &server.url,
        AdminRequest::ListPrograms {
            after: None,
            limit: 256,
        },
    )
    .unwrap();
    assert_eq!(u16::from_le_bytes(programs[..2].try_into().unwrap()), 1);
    assert_eq!(&programs[2..], iid);
}
