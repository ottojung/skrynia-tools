use super::super::{auth, bridge, frame};
use super::fake::Fake;

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

#[test]
fn invalid_path_never_touches_mailbox_or_import() {
    let mut fake = Fake::default();
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

fn request(op: u8, p: &[u8]) -> frame::Request {
    frame::Request {
        publish: op == frame::OP_PUBLISH,
        path: p.to_vec(),
    }
}

#[test]
fn collision_retries_without_overwriting_an_existing_request() {
    let mut fake = Fake {
        collisions: 2,
        ..Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::OK
    );
    assert_eq!(fake.invokes.len(), 1);
    assert_eq!(fake.invokes[0].1, 2);
}

#[test]
fn trap_is_unknown_and_cleanup_failure_is_reported() {
    let mut fake = Fake {
        trap: true,
        ..Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::UNKNOWN
    );
    let mut fake = Fake {
        deny_delete: true,
        ..Fake::default()
    };
    assert_eq!(
        bridge::run(&mut fake, &request(frame::OP_PUBLISH, PUBLIC)),
        bridge::CLEANUP
    );
}

#[test]
fn token_check_is_constant_time_shape_validated() {
    let token = [7u8; 32];
    let mut fake = Fake::with_verifier(&token);
    let mut input = vec![1u8];
    input.extend_from_slice(&token);
    fake.input = input;
    assert_eq!(auth::require(&mut fake, 99).unwrap(), token);
    let mut wrong = Fake::with_verifier(&token);
    wrong.input = [vec![1u8], vec![9u8; 32]].concat();
    assert_eq!(auth::require(&mut wrong, 99), Err(-4));
}
