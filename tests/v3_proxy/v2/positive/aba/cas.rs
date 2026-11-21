use super::{call, fixture};
use sha2::{Digest, Sha256};

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn byte_identity_aba_cas_accepts_restored_digest() {
    let (mut s, token) = fixture::start(None);
    let create = fixture::req("POST", "team/item", &token, b"A");
    assert_eq!(call::call(&s, &create).status, 201);

    let mut to_b = fixture::req("PUT", "team/item", &token, b"B");
    to_b.headers.insert("x-skrynia-if-sha256".into(), sha(b"A"));
    assert_eq!(call::call(&s, &to_b).status, 200);

    let mut to_a = fixture::req("PUT", "team/item", &token, b"A");
    to_a.headers.insert("x-skrynia-if-sha256".into(), sha(b"B"));
    assert_eq!(call::call(&s, &to_a).status, 200);

    let mut again = fixture::req("PUT", "team/item", &token, b"B");
    again
        .headers
        .insert("x-skrynia-if-sha256".into(), sha(b"A"));
    assert_eq!(call::call(&s, &again).status, 200);
    assert_eq!(s.read_file("v2/objects/team/item").unwrap(), b"B");

    s.restart_with_guest(&fixture::guest());
    let read = call::req("GET", "/store/team/item");
    assert_eq!(call::call(&s, &read).body, b"B");
}
