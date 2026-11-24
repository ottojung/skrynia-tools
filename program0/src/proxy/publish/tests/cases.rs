use super::{bridge, fake::Fake, path};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

#[test]
fn ticket_is_eight_lowercase_hex_digits() {
    let p = path::ticket(0x0000_002a);
    assert_eq!(&p, b"publication-requests/0000002a");
    assert_eq!(&path::ticket(u32::MAX), b"publication-requests/ffffffff");
}

#[test]
fn only_bounded_public_relative_utf8_paths_are_accepted() {
    assert!(path::is_public(PUBLIC));
    let bad: [&[u8]; 11] = [
        b"",
        b"/apps/a",
        b"apps/a/../b",
        b"apps/a/./b",
        b"apps/a//b",
        b"apps/a/",
        b"apps/a\0b",
        b"apps/a\nb",
        b"apps/a\\b",
        b"security/program0.sha256",
        b"programs/root.wasm",
    ];
    for p in bad {
        assert!(!path::is_public(p), "accepted unsafe path {:?}", p);
    }
}

#[test]
fn publish_writes_exact_bytes_then_cleans_mailbox() {
    let mut fake = Fake::default();
    assert_eq!(bridge::call(&mut fake, true, PUBLIC), 0);
    assert_eq!(fake.invokes, vec![(true, 0)]);
    assert!(fake.files.is_empty());
    assert_eq!(fake.written, vec![PUBLIC.to_vec()]);
}

#[test]
fn ticket_collision_retries_without_overwriting() {
    let mut fake = Fake {
        collisions: 2,
        ..Fake::default()
    };
    assert_eq!(bridge::call(&mut fake, false, PUBLIC), 0);
    assert_eq!(fake.invokes, vec![(false, 2)]);
    assert!(fake.files.is_empty());
}
