use super::{Server, cases};

fn denied(res: reqwest::blocking::Response) {
    assert_eq!(res.status(), 500);
    assert!(res.bytes().unwrap().is_empty());
}
fn rename_outside() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/rename_outside.wat")).unwrap()
}
fn read_security() -> Vec<u8> {
    wat::parse_str(include_str!("../wat/read_security.wat")).unwrap()
}

#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_rename_outside_fixture() {
    let s = Server::start(Some(&rename_outside()));
    s.seed_file("outside/secret.txt", b"outside-secret");
    s.seed_file("granted/.keep", b"");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
    assert_eq!(
        s.read_file("outside/secret.txt").unwrap(),
        b"outside-secret"
    );
    assert_eq!(s.read_file("granted/moved.txt"), None);
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_read_private_verifier() {
    let s = Server::start(Some(&read_security()));
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
}
