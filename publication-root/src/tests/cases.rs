use super::super::{bridge, frame, path};
use super::fake::Fake;

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

fn request(op: u8, p: &[u8]) -> frame::Request {
    frame::Request {
        publish: op == frame::OP_PUBLISH,
        path: p.to_vec(),
    }
}

#[test]
fn public_path_grammar_and_private_roots() {
    assert!(path::valid(PUBLIC));
    let bad: [&[u8]; 10] = [
        b"",
        b"/apps/a",
        b"apps/a/../b",
        b"apps/a/./b",
        b"apps/a//b",
        b"apps/a/",
        b"apps/a\0b",
        b"apps/a\\b",
        b"security/program0.sha256",
        b"programs/root.wasm",
    ];
    for p in bad {
        assert!(!path::valid(p), "accepted unsafe path {:?}", p);
    }
    assert_eq!(&path::mailbox(0x2a), b"publication-requests/0000002a");
}

#[test]
fn frame_is_exact_length_prefixed_path() {
    let mut raw = vec![0u8; 34];
    raw[0] = frame::VERSION;
    raw[33] = frame::OP_UNPUBLISH;
    raw.extend_from_slice(&(PUBLIC.len() as u32).to_le_bytes());
    raw.extend_from_slice(PUBLIC);
    let decoded = frame::decode(&raw).unwrap();
    assert!(!decoded.publish);
    assert_eq!(decoded.path, PUBLIC);
    assert!(frame::decode(&raw[..raw.len() - 1]).is_none());
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
