use super::{ERR_TICKET, ERR_VALIDATION, bridge, fake::Fake, wire};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

#[test]
fn provider_code_is_passed_through_and_mailbox_still_cleaned() {
    let mut fake = Fake {
        provider: 4,
        ..Fake::default()
    };
    assert_eq!(bridge::call(&mut fake, true, PUBLIC), 4);
    assert!(fake.files.is_empty());
    assert_eq!(fake.invokes.len(), 1);
}

#[test]
fn private_path_is_rejected_before_any_host_call() {
    let mut fake = Fake::default();
    assert_eq!(
        bridge::call(&mut fake, true, b"security/program0.sha256"),
        ERR_VALIDATION
    );
    assert!(fake.invokes.is_empty());
    assert!(fake.files.is_empty());
}

#[test]
fn wire_request_carries_length_and_exact_path() {
    let req = wire::request(wire::OP_UNPUBLISH, PUBLIC);
    assert_eq!(req[0], wire::OP_UNPUBLISH);
    assert_eq!(&req[1..5], &(PUBLIC.len() as u32).to_le_bytes());
    assert_eq!(&req[5..], PUBLIC);
}

#[test]
fn persistent_collision_fails_closed_without_a_request() {
    let mut fake = Fake {
        collisions: 255,
        ..Fake::default()
    };
    assert_eq!(bridge::call(&mut fake, true, PUBLIC), ERR_TICKET);
    assert!(fake.invokes.is_empty());
    assert!(fake.files.is_empty());
}
