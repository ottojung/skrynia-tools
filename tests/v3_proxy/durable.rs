use super::{cases, fixture::Server, guest};
#[test]
#[ignore = "requires independently compiled Rust Program0 and fixed generic FD kernel"]
fn generic_syscall_relay_persists_atomic_replace_and_unlink() {
    let mut s = Server::start(Some(&guest::durable()));
    s.seed_file("records/public.txt", b"old");
    s.seed_file("records/obsolete", b"old");
    let denied = cases::post(&s.url, &cases::authenticated(b"go", &[0x99; 32]));
    assert_eq!(denied.status(), 500);
    assert_eq!(s.read_file("records/public.txt").unwrap(), b"old");
    assert_eq!(s.read_file("records/obsolete").unwrap(), b"old");
    let ok = cases::post(&s.url, &cases::authenticated(b"go", &[0x42; 32]));
    assert_eq!(ok.status(), 200);
    assert_eq!(ok.bytes().unwrap().as_ref(), b"OK");
    assert_eq!(s.read_file("records/public.txt").unwrap(), b"durable");
    assert!(s.read_file("records/obsolete").is_none());
    assert!(s.read_file("records/staged").is_none());
    assert!(s.dir_exists("records/new"));
    // This verifies an independently started kernel opens the persisted
    // file; it does not simulate power loss or establish CAS semantics.
    s.restart_with_guest(&guest::read_disk());
    let response = cases::post(&s.url, &cases::authenticated(b"read", &[0x42; 32]));
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().unwrap().as_ref(), b"durable");
    assert!(s.read_file("records/obsolete").is_none());
}
