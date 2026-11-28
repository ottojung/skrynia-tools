//! Ticket-open error classification against the pinned ABI 3.1 kernel.
use super::{Fake, bridge, frame};

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
