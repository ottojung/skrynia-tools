use super::{cases, fixture::Server, guest};

#[test]
#[ignore = "requires actual Rust Program0 with lock relay and generic kernel PR #112"]
fn ordinary_guest_uses_request_owned_generic_lock_via_callback() {
    let s = Server::start(Some(&guest::mutation_lock()));
    let invalid = cases::post(&s.url, &cases::authenticated(b"root", &[0x99; 32]));
    assert_eq!(invalid.status(), 500);
    let first = cases::post(&s.url, &cases::authenticated(b"root", &[0x42; 32]));
    assert_eq!(first.status(), 200);
    assert_eq!(first.bytes().unwrap().as_ref(), b"LOCKED");
    let again = cases::post(&s.url, &cases::authenticated(b"root", &[0x42; 32]));
    assert_eq!(again.status(), 200);
    assert_eq!(again.bytes().unwrap().as_ref(), b"LOCKED");
}
#[test]
#[ignore = "requires actual Rust Program0 with lock relay and generic kernel PR #112"]
fn lock_is_released_after_unprivileged_guest_trap() {
    let s = Server::start(Some(&guest::mutation_trap()));
    let trapped = cases::post(&s.url, &cases::authenticated(b"x", &[0x42; 32]));
    assert_eq!(trapped.status(), 500);
    s.seed_file("programs/root.wasm", &guest::mutation_lock());
    let next = cases::post(&s.url, &cases::authenticated(b"y", &[0x42; 32]));
    assert_eq!(next.status(), 200);
    assert_eq!(next.bytes().unwrap().as_ref(), b"LOCKED");
}
#[test]
#[ignore = "requires actual Rust Program0 with lock relay and generic kernel PR #112"]
fn nested_child_reuses_parent_generic_mutation_lock_callback() {
    let s = Server::start(Some(&guest::nested_router()));
    s.seed_file("programs/nested.wasm", &guest::mutation_lock());
    let response = cases::post(&s.url, &cases::authenticated(b"!nested", &[0x42; 32]));
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().unwrap().as_ref(), b"LOCKED");
}
#[test]
#[ignore = "requires actual Rust Program0 with lock relay and generic kernel PR #112"]
fn guest_may_not_import_privileged_kernel_lock_directly() {
    let s = Server::start(Some(&guest::forbidden_lock_import()));
    let response = cases::post(&s.url, &cases::authenticated(b"test", &[0x42; 32]));
    assert_eq!(response.status(), 500);
    s.seed_file("programs/root.wasm", &guest::mutation_lock());
    let again = cases::post(&s.url, &cases::authenticated(b"test", &[0x42; 32]));
    assert_eq!(again.status(), 200);
}
