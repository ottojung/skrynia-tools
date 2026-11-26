use super::{Fake, auth, bridge, frame, path};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

fn credential(token: &[u8; 32]) -> Fake {
    let mut fake = Fake::with_verifier(token);
    let mut input = vec![1u8];
    input.extend_from_slice(token);
    fake.input = input;
    fake
}

#[test]
fn root_token_rejects_bad_version_short_header_and_long_verifier() {
    let token = [3u8; 32];
    let mut bad_version = credential(&token);
    bad_version.input[0] = 2;
    assert_eq!(auth::require(&mut bad_version, 1), Err(-4));
    let mut short = credential(&token);
    short.input.truncate(10);
    assert_eq!(auth::require(&mut short, 1), Err(-4));
    let mut long_verifier = credential(&token);
    long_verifier.verifier_extra.push(0);
    assert_eq!(auth::require(&mut long_verifier, 1), Err(-4));
    assert_eq!(auth::require(&mut credential(&token), 1).unwrap(), token);
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

#[test]
fn path_security_rejects_control_characters_and_overlong_paths() {
    for p in [
        &b"apps/a\nb"[..],
        b"apps/a\rb",
        b"apps/a\0b",
        b"\\apps",
        b"apps//b",
        b"apps/a/../b",
    ] {
        assert!(!path::valid(p), "accepted {:?}", p);
    }
    assert!(path::valid(b"apps/a-b_c.1/releases/abcdef/index.html"));
    assert!(!path::valid(&vec![b'a'; 4097]));
    assert_eq!(&path::mailbox(0), b"publication-requests/00000000");
    assert_eq!(&path::mailbox(u32::MAX), b"publication-requests/ffffffff");
}
