use super::{Server, cases};

fn denied(res: reqwest::blocking::Response) {
    assert_eq!(res.status(), 500);
    assert!(res.bytes().unwrap().is_empty());
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_read_outside_with_scope() {
    let g = wat::parse_str(include_str!("../wat/adv/read_outside.wat")).unwrap();
    let s = Server::start(Some(&g));
    s.grant_all_userland("userland");
    s.seed_file("outside/secret.txt", b"outside-secret");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
    assert_eq!(
        s.read_file("outside/secret.txt").unwrap(),
        b"outside-secret"
    );
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_write_outside_with_scope() {
    let g = wat::parse_str(include_str!("../wat/adv/write_outside.wat")).unwrap();
    let s = Server::start(Some(&g));
    s.grant_all_userland("userland");
    s.seed_file("outside/existing.txt", b"keep");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
    assert_eq!(s.read_file("outside/evil.bin"), None);
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn missing_scope_denies_privileged_callbacks() {
    let g = wat::parse_str(include_str!("../wat/adv/read_outside.wat")).unwrap();
    let s = Server::start(Some(&g));
    s.seed_file("outside/secret.txt", b"outside-secret");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
    assert_eq!(
        s.read_file("outside/secret.txt").unwrap(),
        b"outside-secret"
    );
}
