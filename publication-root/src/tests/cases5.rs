use super::{Fake, bridge, frame, path};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

fn request(op: u8, p: &[u8]) -> frame::Request {
    frame::Request {
        publish: op == frame::OP_PUBLISH,
        path: p.to_vec(),
    }
}

#[test]
fn write_failure_cleans_only_created_ticket_not_preexisting() {
    // Ticket 0 collides (pre-existing, must never be deleted); ticket 1 is
    // created by this call and its write fails before any import.
    let mut fake = Fake {
        collisions: 1,
        fail_write: true,
        ..Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::MAILBOX
    );
    assert!(fake.invokes.is_empty());
    assert_eq!(fake.deleted, vec![path::mailbox(1)]);
    assert!(!fake.deleted.contains(&path::mailbox(0)));
}

#[test]
fn close_failure_cleans_created_ticket() {
    let mut fake = Fake {
        fail_close: true,
        ..Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::MAILBOX
    );
    assert!(fake.invokes.is_empty());
    assert_eq!(fake.deleted, vec![path::mailbox(0)]);
}
