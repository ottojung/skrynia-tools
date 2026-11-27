use super::{Fake, bridge, frame};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

fn request(op: u8, p: &[u8]) -> frame::Request {
    frame::Request {
        publish: op == frame::OP_PUBLISH,
        path: p.to_vec(),
    }
}

#[test]
fn publish_is_synchronous_and_cleans_mailbox() {
    let mut fake = Fake::default();
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::OK
    );
    assert_eq!(fake.invokes, vec![(true, 0)]);
    assert_eq!(fake.written, vec![PUBLIC.to_vec()]);
    assert!(fake.files.is_empty());
}

#[test]
fn provider_code_passes_through_and_mailbox_is_cleaned() {
    let mut fake = Fake {
        provider: 4,
        ..Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_UNPUBLISH, PUBLIC)),
        4
    );
    assert_eq!(fake.invokes, vec![(false, 0)]);
    assert!(fake.files.is_empty());
}

#[test]
fn random_failure_never_invokes_the_provider() {
    let mut fake = Fake {
        fail_random: true,
        ..Fake::default()
    };
    let request = frame::Request {
        publish: true,
        path: PUBLIC.to_vec(),
    };
    assert_eq!(bridge::run(&mut fake, &request), bridge::MAILBOX);
    assert!(fake.invokes.is_empty());
    assert!(fake.written.is_empty());
}
