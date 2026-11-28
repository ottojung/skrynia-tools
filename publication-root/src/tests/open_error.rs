//! Host error classification against the pinned ABI 3.1 kernel.
use super::{Fake, auth, bridge, frame};

fn request() -> frame::Request {
    frame::Request {
        publish: true,
        path: b"apps/hello/index.html".to_vec(),
    }
}

#[test]
fn eexist_from_create_only_open_is_retried() {
    let mut fake = Fake {
        collisions: 2,
        ..Fake::default()
    };
    assert_eq!(bridge::run(&mut fake, &request()), bridge::OK);
    assert_eq!(fake.invokes.len(), 1);
    assert_eq!(fake.invokes[0].1, 2);
}

#[test]
fn einval_from_open_is_not_a_ticket_collision() {
    let mut fake = Fake {
        open_error: Some(-3),
        ..Fake::default()
    };
    assert_eq!(bridge::run(&mut fake, &request()), bridge::MAILBOX);
    assert!(fake.invokes.is_empty());
}

#[test]
fn noncollision_eio_never_retries_invokes_or_deletes() {
    let mut fake = Fake {
        open_error: Some(-5),
        ..Fake::default()
    };
    assert_eq!(bridge::run(&mut fake, &request()), bridge::MAILBOX);
    assert!(fake.invokes.is_empty());
    assert!(fake.deleted.is_empty());
}

#[test]
fn auth_host_errnos_are_never_ambiguous_response_codes() {
    for code in [-3, -4, -5, -8] {
        let mut fake = super::with_io_error(Some(code), None);
        assert_eq!(auth::require(&mut fake, 99), Err(-4), "read errno {code}");
    }
    let mut fake = super::with_io_error(None, Some(-8));
    assert_eq!(auth::require(&mut fake, 99), Err(-4));
}
