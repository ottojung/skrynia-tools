use super::{bridge, frame};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

fn request(op: u8, p: &[u8]) -> frame::Request {
    frame::Request {
        publish: op == frame::OP_PUBLISH,
        path: p.to_vec(),
    }
}

#[test]
fn invalid_path_never_touches_mailbox_or_import() {
    let mut fake = super::Fake::default();
    assert_eq!(
        bridge::run(
            &mut fake,
            &request(frame::OP_PUBLISH, b"security/program0.sha256")
        ),
        bridge::INVALID
    );
    assert!(fake.invokes.is_empty());
    assert!(fake.written.is_empty());
}

#[test]
fn collision_retries_without_overwriting_an_existing_request() {
    let mut fake = super::Fake {
        collisions: 2,
        ..super::Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::OK
    );
    assert_eq!(fake.invokes.len(), 1);
    assert_eq!(fake.invokes[0].1, 2);
}
