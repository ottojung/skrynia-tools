use super::{call, cases, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};
use std::{
    sync::{Arc, Barrier},
    thread,
};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn concurrent_child_grant_creation_persists_all_grants() {
    let s = Server::start(Some(&fixture::guest()));
    let (issuer, user) = ("1".repeat(64), "2".repeat(64));
    let g = Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "delegate".into()],
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
    let gate = Arc::new(Barrier::new(3));
    let hs: Vec<_> = (0..2).map(|n| {
        let (gate, url, token) = (gate.clone(), s.url.clone(), user.clone());
        thread::spawn(move || {
            let mut req = call::req("POST", "/grants");
            req.headers.insert("authorization".into(), format!("Bearer {token}"));
            req.body = format!(r#"{{"locations":"team/{n}/*","operations":["create"],"maxBytes":4096,"maxObjects":5}}"#).into();
            let body = serde_json::to_vec(&req).unwrap();
            gate.wait();
            let http = cases::post(&url, &cases::authenticated(&body, &[0x42; 32]));
            let ans: skrynia_tools::wire::v2compat::Response = serde_json::from_slice(&http.bytes().unwrap()).unwrap();
            ans.status
        })
    }).collect();
    gate.wait();
    assert!(hs.into_iter().all(|h| h.join().unwrap() == 200));
    assert!(s.directory_names("v2/state/grants").len() >= 3);
}
