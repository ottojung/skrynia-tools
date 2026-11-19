use super::{process, run, wire};
use skrynia_program0::wire::V2Kind;
pub fn verify(server: &process::Server, token: [u8; 32]) {
    let key = b"corrupt";
    let name = hex::encode(wire::sha256(key));
    let path = server.data_dir().join("v2auth/objs").join(name);
    std::fs::write(&path, b"{corrupt").unwrap();
    let create = run::req(token, V2Kind::Create, key, None, b"unwanted");
    assert_eq!(wire::err(&server.url, create).unwrap(), "state_uncertain");
    assert_eq!(std::fs::read(&path).unwrap(), b"{corrupt");
    let invalid = run::req(token, V2Kind::Create, b"bad//x", None, b"wrong");
    assert_eq!(wire::err(&server.url, invalid).unwrap(), "invalid_key");
}
