use skrynia_publish_provider::{mailbox, path};

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
