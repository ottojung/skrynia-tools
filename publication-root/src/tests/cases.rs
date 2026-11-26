use super::{frame, path};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

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
    assert_eq!(&path::mailbox(0x2a), b"user/publication-requests/0000002a");
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
