use super::{cases, fixture, guest};
use fixture::Server;
mod adversarial;
mod adversarial_more;
mod adversarial_scope;
#[test]
#[ignore = "requires actual compiled Program0 and callback-safe generic kernel"]
fn generic_guest_reads_private_disk_only_via_parent_api_callback() {
    let s = Server::start(Some(&guest::read_disk()));
    s.grant_all_userland("userland");
    s.seed_file("userland/records/public.txt", b"Hello from v3 disk");
    let authenticated = cases::authenticated(b"opaque", &[0x42; 32]);
    let res = cases::post(&s.url, &authenticated);
    assert_eq!(res.status(), 200);
    let bytes = res.bytes().unwrap();
    assert_eq!(
        bytes.as_ref(),
        b"Hello from v3 disk",
        "debug error = {:?}",
        bytes
    );
    let unauthenticated = cases::post(&s.url, &cases::authenticated(b"opaque", &[0x99; 32]));
    assert_eq!(unauthenticated.status(), 500);
    assert!(unauthenticated.bytes().unwrap().is_empty());
}

#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn generic_host_api_callback_can_create_and_write_a_disk_file() {
    let s = Server::start(Some(&guest::write_disk()));
    s.grant_all_userland("userland");
    s.seed_file("userland/records/existing", b"old");
    let bad = cases::authenticated(b"request", &[0x99; 32]);
    assert_eq!(cases::post(&s.url, &bad).status(), 500);
    assert!(s.read_file("userland/records/new.bin").is_none());
    let good = cases::authenticated(b"request", &[0x42; 32]);
    let response = cases::post(&s.url, &good);
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().unwrap().as_ref(), b"OK");
    assert_eq!(
        s.read_file("userland/records/new.bin").unwrap(),
        b"disk-write"
    );
}
