//! Fail-closed behavior on hostile paths and malformed tickets.

mod support;

use support::{Fixture, call, code};

#[test]
fn rejects_bad_tickets_and_paths() {
    let f = Fixture::start();
    assert_eq!(call(&f, "publish", 7), 1);
    let long = vec![b'a'; 4097];
    for path in [
        &b"../escape"[..],
        &b"/abs"[..],
        &b"a//b"[..],
        &b"a/"[..],
        &b"x\0y"[..],
        &b""[..],
        &long,
    ] {
        assert_eq!(code(&f, "publish", 8, path), 2);
    }
}
