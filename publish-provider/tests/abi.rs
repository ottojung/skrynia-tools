//! Pure ABI and path-grammar checks, independent of ambient I/O.

use skrynia_publish_provider::{abi, mailbox, path};

#[test]
fn typed_i32_little_endian_codec_is_exact() {
    assert_eq!(abi::decode(&[1, 1, 0x32, 4, 3, 2, 1]), Ok(0x0102_0304));
    assert_eq!(
        abi::encode(abi::DEST_CONFLICT),
        vec![1, 1, 0x32, 4, 0, 0, 0]
    );
    for value in [0, 1, 0x8000_0000, u32::MAX] {
        assert_eq!(abi::decode(&abi::encode(value)), Ok(value));
    }
    for bad in [
        &[][..],
        &[0, 0, 0, 0],
        &[2, 1, 0x32, 4, 3, 2, 1],
        &[1, 0, 0x32, 4, 3, 2, 1],
        &[1, 1, 0x40, 4, 3, 2, 1, 0, 0, 0, 0],
        &[1, 1, 0x32, 4, 3, 2],
        &[1, 1, 0x32, 4, 3, 2, 1, 0],
    ] {
        assert_eq!(abi::decode(bad), Err(abi::BAD_TICKET));
    }
}

#[test]
fn path_grammar_matches_kernel_clean_relative() {
    assert_eq!(
        path::clean(b"apps/a/./b"),
        Ok(vec!["apps".into(), "a".into(), "b".into()])
    );
    assert_eq!(
        path::clean(b"example/path/a"),
        Ok(vec!["example".into(), "path".into(), "a".into()])
    );
    for bad in [
        &b""[..],
        &b"/abs"[..],
        &b"../up"[..],
        &b"a/../b"[..],
        &b"a//b"[..],
        &b"a/"[..],
        &b"a\0b"[..],
        &b"\xff"[..],
        &b"a\\b"[..],
        &b"a\nb"[..],
        &b"publication-requests/00000001"[..],
    ] {
        assert!(path::clean(bad).is_err(), "accepted {bad:?}");
    }
    let long = vec![b'a'; 4097];
    assert!(path::clean(&long).is_err());
}

#[test]
fn ticket_name_is_fixed_lowercase_hex() {
    assert_eq!(mailbox::name(42), "0000002a");
    assert_eq!(mailbox::name(u32::MAX), "ffffffff");
}

#[test]
fn syscalls_list_advertises_both_names() {
    assert_eq!(
        abi::syscalls_list(),
        serde_json::json!({"functions": [
            {"name": "publish", "params": ["i32"], "results": ["i32"]},
            {"name": "unpublish", "params": ["i32"], "results": ["i32"]}
        ]})
    );
}
