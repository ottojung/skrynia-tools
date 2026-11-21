use super::scope::{Fds, MAX_FDS, path_in, write_in};

#[test]
fn delegated_path_is_descendant_only() {
    assert!(path_in("programs", b"programs"));
    assert!(path_in("programs", b"programs/app/a.json"));
    for bad in [
        b"programs2/a".as_slice(),
        b"../programs/a",
        b"programs/../secret",
        b"programs//a",
        b"/programs/a",
        b"programs/./a",
        b"programs/a\0b",
        &[0xff, 0xfe],
    ] {
        assert!(!path_in("programs", bad), "accepted {bad:?}");
    }
    assert!(!path_in("", b"programs/a"));
}

#[test]
fn write_authority_is_explicit_and_narrow() {
    assert!(!write_in("programs", false, b"programs/a"));
    assert!(write_in("programs", true, b"programs/a"));
    assert!(!write_in("programs", true, b"other/a"));
}

#[test]
fn fd_authority_is_granted_bounded_and_revocable() {
    let mut fds = Fds::new();
    assert!(!fds.has(7));
    assert!(fds.grant(7));
    assert!(fds.has(7));
    assert!(!fds.grant(7));
    assert!(!fds.grant(-1));
    for fd in 100..100 + MAX_FDS as i32 - 1 {
        assert!(fds.grant(fd));
    }
    assert!(!fds.grant(1000));
    fds.revoke(7);
    assert!(!fds.has(7));
}
