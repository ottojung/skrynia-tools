use super::{Fake, auth, bridge, frame};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

fn request(op: u8, p: &[u8]) -> frame::Request {
    frame::Request {
        publish: op == frame::OP_PUBLISH,
        path: p.to_vec(),
    }
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
fn token_check_is_shape_validated() {
    let token = [7u8; 32];
    let mut fake = super::with_verifier(&token);
    let mut input = vec![1u8];
    input.extend_from_slice(&token);
    fake.input = input;
    assert_eq!(auth::require(&mut fake, 99).unwrap(), token);
    let mut wrong = super::with_verifier(&token);
    wrong.input = [vec![1u8], vec![9u8; 32]].concat();
    assert_eq!(auth::require(&mut wrong, 99), Err(-4));
}
