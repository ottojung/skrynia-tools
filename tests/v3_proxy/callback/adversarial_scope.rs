use super::{Server, cases};

fn denied(res: reqwest::blocking::Response) {
    assert_eq!(res.status(), 500);
    assert!(res.bytes().unwrap().is_empty());
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_use_raw_fd_guess() {
    let g = wat::parse_str(include_str!("../wat/adv/read_fd_guess.wat")).unwrap();
    let s = Server::start(Some(&g));
    s.grant_all_userland("userland");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
}
#[test]
#[ignore = "requires compiled Program0 and callback-safe generic kernel"]
fn guest_cannot_traverse_to_other_scope() {
    let g = wat::parse_str(include_str!("../wat/adv/read_traversal.wat")).unwrap();
    let s = Server::start(Some(&g));
    s.grant_all_userland("userland/a");
    s.seed_file("userland/b/secret.txt", b"other-scope");
    denied(cases::post(
        &s.url,
        &cases::authenticated(b"x", &[0x42; 32]),
    ));
    assert_eq!(
        s.read_file("userland/b/secret.txt").unwrap(),
        b"other-scope"
    );
}
