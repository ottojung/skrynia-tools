use super::run::req;
use super::{process, wire};
use skrynia_program0::wire::V2Kind;

pub fn finish(server: &mut process::Server, token: [u8; 32]) {
    server.restart().unwrap();
    let good = req(
        token,
        V2Kind::Replace,
        b"a.txt",
        Some(wire::sha256(b"v1")),
        b"v2",
    );
    wire::ok(&server.url, good).unwrap();
    let wrong = req(
        [9u8; 32],
        V2Kind::Replace,
        b"a.txt",
        Some(wire::sha256(b"v2")),
        b"v3",
    );
    assert_eq!(
        wire::err(&server.url, wrong).unwrap(),
        "invalid_grant_user_token"
    );
    super::corrupt::verify(server, token);
    wire::mark_uncertain(server.data_dir());
    let after = req(
        token,
        V2Kind::Replace,
        b"a.txt",
        Some(wire::sha256(b"v2")),
        b"v3",
    );
    assert_eq!(wire::err(&server.url, after).unwrap(), "state_uncertain");
}
