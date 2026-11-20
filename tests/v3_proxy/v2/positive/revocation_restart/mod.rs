mod grant;
mod user;

pub(super) use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

fn root_server(delegate: bool) -> (Server, String, String) {
    let s = Server::start(Some(&fixture::guest()));
    let (issuer, user) = ("1".repeat(64), "2".repeat(64));
    let mut operations = vec!["create".into()];
    if delegate {
        operations.push("delegate".into());
    }
    let g = Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations,
        max_bytes: None,
        max_objects: None,
        issuer_hash: hex::encode(Sha256::digest(issuer.as_bytes())),
        root_issuer_token: None,
        users: vec![User {
            id: "u1".into(),
            hash: hex::encode(Sha256::digest(user.as_bytes())),
        }],
    };
    s.seed_file(
        "v2/state/grants/root.json",
        &serde_json::to_vec(&g).unwrap(),
    );
    (s, issuer, user)
}
