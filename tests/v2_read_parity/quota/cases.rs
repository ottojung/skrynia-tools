use super::{RootStore, compare};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::Request;
use std::collections::BTreeMap;
pub fn run(url: &str, token: &str, store: &RootStore) -> Result<(), String> {
    let cases = [
        ("POST", "team/one", vec![b'a'], None),
        ("POST", "team/two", vec![b'b'], None),
        ("PUT", "team/one", vec![b'a'; 500], None),
        ("PUT", "team/one", b"abc".to_vec(), None),
        ("PUT", "team/one", vec![b'a'; 500], Some("00".repeat(32))),
        (
            "PUT",
            "team/one",
            vec![b'a'; 500],
            Some(hex::encode(Sha256::digest(b"abc"))),
        ),
        ("POST", "team/two", vec![b'b'], None),
        ("DELETE", "team/one", Vec::new(), None),
        ("POST", "team/two", vec![b'b'], None),
        ("PUT", "team/two", vec![b'b'; 500], None),
        ("POST", "other/one", vec![b'a'], None),
    ];
    for (method, key, bytes, expected) in cases {
        let mut headers = BTreeMap::new();
        headers.insert("authorization".into(), format!("Bearer {token}"));
        if let Some(expected) = expected {
            headers.insert("x-skrynia-if-sha256".into(), expected);
        }
        let request = Request {
            method: method.into(),
            path: format!("/store/{key}"),
            query: None,
            headers,
            body: bytes,
        };
        compare::compare(url, store, request)?;
    }
    Ok(())
}
