use super::{call, fixture};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn committed_mutations_survive_restart_without_staging_artifacts() {
    let (mut s, token) = fixture::start(None);
    for (i, body) in [b"one".as_slice(), b"two", b"three"]
        .into_iter()
        .enumerate()
    {
        let req = fixture::req("POST", &format!("team/{i}"), &token, body);
        assert_eq!(call::call(&s, &req).status, 201);
    }
    let names = s.directory_names("v2/objects/team");
    assert!(names.iter().all(|n| !n.contains(".tmp.")), "{names:?}");

    s.restart_with_guest(&fixture::guest());
    for (i, body) in [b"one".as_slice(), b"two", b"three"]
        .into_iter()
        .enumerate()
    {
        let read = call::req("GET", &format!("/store/team/{i}"));
        assert_eq!(call::call(&s, &read).body, body);
    }
    let names = s.directory_names("v2/objects/team");
    assert_eq!(names.len(), 3);
    assert!(names.iter().all(|n| !n.contains(".tmp.")), "{names:?}");
}
