use super::{call, fixture};
use sha2::{Digest, Sha256};
#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn v2_authorized_create_replace_delete_with_sha_cas_survive_restart() {
    let (mut s, token) = fixture::start(None);
    let create = fixture::req("POST", "team/new", &token, b"one");
    assert_eq!(call::call(&s, &create).status, 201);
    assert_eq!(s.read_file("v2/objects/team/new").unwrap(), b"one");
    let mut replace = fixture::req("PUT", "team/new", &token, b"two");
    replace.headers.insert(
        "x-skrynia-if-sha256".into(),
        hex::encode(Sha256::digest(b"one")),
    );
    assert_eq!(call::call(&s, &replace).status, 200);
    assert_eq!(s.read_file("v2/objects/team/new").unwrap(), b"two");
    let names = s.directory_names("v2/objects/team");
    assert_eq!(names, ["new"]);

    s.restart_with_guest(&fixture::guest());
    let read = call::req("GET", "/store/team/new");
    assert_eq!(call::call(&s, &read).body, b"two");
    let del = fixture::req("DELETE", "team/new", &token, b"");
    assert_eq!(call::call(&s, &del).status, 200);
    assert!(s.read_file("v2/objects/team/new").is_none());
    s.restart_with_guest(&fixture::guest());
    assert_eq!(call::call(&s, &read).status, 404);
}
