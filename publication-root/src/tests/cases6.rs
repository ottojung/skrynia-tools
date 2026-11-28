use super::{Fake, auth, path};

fn credential(token: &[u8; 32]) -> Fake {
    let mut fake = super::with_verifier(token);
    fake.input = [vec![1u8], token.to_vec()].concat();
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
fn path_security_rejects_control_characters_and_overlong_paths() {
    for p in [
        &b"apps/a\nb"[..],
        b"apps/a\rb",
        b"apps/a\0b",
        b"\\apps",
        b"apps//b",
        b"apps/a/../b",
        b"user/publication-requests/00000001",
        b"user/security/program0.sha256",
        b"user/private/key",
        b"operator/security/program0.sha256",
    ] {
        assert!(!path::valid(p), "accepted {:?}", p);
    }
    assert!(path::valid(b"apps/a-b_c.1/releases/abcdef/index.html"));
    assert!(!path::valid(&vec![b'a'; 4097]));
    assert_eq!(&path::mailbox(0), b"user/publication-requests/00000000");
    assert_eq!(
        &path::mailbox(u32::MAX),
        b"user/publication-requests/ffffffff"
    );
}
