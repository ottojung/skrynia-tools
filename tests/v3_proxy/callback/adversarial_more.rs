use super::{Server, cases};

fn denied(res: reqwest::blocking::Response) {
    assert_eq!(res.status(), 500);
    assert!(res.bytes().unwrap().is_empty());
}
fn rename_outside() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/adv/rename_outside.wat")).unwrap()
}
fn read_security() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/adv/read_security.wat")).unwrap()
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_delete_or_rename_outside_with_scope() {
    let del = wat::parse_str(include_str!("../wat/adv/delete_outside.wat")).unwrap();
    let s = Server::start(Some(&del));
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
    let s2 = Server::start(Some(&rename_outside()));
    s2.grant_all_userland("userland");
    s2.seed_file("outside/secret.txt", b"outside-secret");
    s2.seed_file("userland/granted/.keep", b"");
    denied(cases::post(
        &s2.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
    assert_eq!(
        s2.read_file("outside/secret.txt").unwrap(),
        b"outside-secret"
    );
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_read_private_verifier_with_scope() {
    let s = Server::start(Some(&read_security()));
    s.grant_all_userland("userland");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
}
