use super::{call, fixture};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn rename_failure_cleans_staging_without_partial_commit() {
    let (mut s, token) = fixture::start(None);
    s.seed_file("v2/objects/team/conflict/.keep", b"x");
    let put = fixture::req("PUT", "team/conflict", &token, b"new");
    assert!(call::call(&s, &put).status >= 500);

    let names = s.directory_names("v2/objects/team/conflict");
    assert_eq!(names, [".keep"]);
    assert!(s.read_file("v2/objects/team/conflict/.keep").is_some());

    let root = s.directory_names("v2/objects/team");
    assert!(root.contains(&"conflict".to_string()));
    assert!(root.iter().all(|n| !n.contains(".tmp.")), "{root:?}");
    assert!(!s.dir_exists("v2/objects/team/conflict.tmp"));

    s.restart_with_guest(&fixture::guest());
    let read = call::req("GET", "/store/team/conflict");
    assert_ne!(call::call(&s, &read).status, 200);
}
