use super::{cases, fixture::Server, guest};

#[test]
#[ignore = "requires compiled generic callback Program0 and security-fixed kernel"]
fn ordinary_router_delegates_two_fds_to_another_arbitrary_guest() {
    let s = Server::start(Some(&guest::nested_router()));
    let input = b"!arbitrary\0binary\xffpost\0";
    let bad = cases::post(&s.url, &cases::authenticated(input, &[0x99; 32]));
    assert_eq!(bad.status(), 500);
    assert!(bad.bytes().unwrap().is_empty());
    s.seed_file("programs/nested.wasm", b"invalid WASM");
    assert_eq!(
        cases::post(&s.url, &cases::authenticated(input, &[0x42; 32])).status(),
        500
    );
    s.seed_file("programs/nested.wasm", &guest::privileged());
    assert_eq!(
        cases::post(&s.url, &cases::authenticated(input, &[0x42; 32])).status(),
        500
    );
    s.seed_file("programs/nested.wasm", &guest::echo());
    let result = cases::post(&s.url, &cases::authenticated(input, &[0x42; 32]));
    assert_eq!(result.status(), 200);
    assert_eq!(result.bytes().unwrap().as_ref(), &input[1..]);
}

#[test]
#[ignore = "requires independently built WASM and security-fixed kernel"]
fn nested_guest_can_read_disk_through_generic_callback() {
    let s = Server::start(Some(&guest::nested_router()));
    s.seed_file("programs/nested.wasm", &guest::read_disk());
    s.seed_file("records/public.txt", b"nested-service-data");
    let result = cases::post(&s.url, &cases::authenticated(b"!payload", &[0x42; 32]));
    assert_eq!(result.status(), 200);
    assert_eq!(result.bytes().unwrap().as_ref(), b"nested-service-data");
}
