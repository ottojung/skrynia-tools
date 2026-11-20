use super::{call, fixture};
#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn v2_grant_scope_token_cas_and_quota_denials_never_mutate_disk() {
    let (s, token) = fixture::start(Some(520));
    let bad = fixture::req("POST", "team/new", &"b".repeat(64), b"x");
    assert_eq!(call::call(&s, &bad).status, 403);
    let outside = fixture::req("POST", "other/out", &token, b"x");
    assert_eq!(call::call(&s, &outside).status, 403);
    let quota = fixture::req("POST", "team/large", &token, &[b'x'; 32]);
    assert_eq!(call::call(&s, &quota).status, 507);
    assert!(s.read_file("v2/objects/team/large").is_none());
    s.seed_file("v2/objects/team/existing", b"old");
    let mut stale = fixture::req("PUT", "team/existing", &token, b"change");
    stale
        .headers
        .insert("x-skrynia-if-sha256".into(), "0".repeat(64));
    assert_eq!(call::call(&s, &stale).status, 412);
    assert_eq!(s.read_file("v2/objects/team/existing").unwrap(), b"old");
}
#[cfg(unix)]
#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn unreadable_object_must_not_be_treated_as_missing_for_create() {
    let (s, token) = fixture::start(None);
    s.seed_file("v2/objects/team/secret", b"original");
    s.set_file_mode("v2/objects/team/secret", 0o000);
    let req = fixture::req("POST", "team/secret", &token, b"attacker");
    assert_ne!(call::call(&s, &req).status, 201);
    s.set_file_mode("v2/objects/team/secret", 0o600);
    assert_eq!(s.read_file("v2/objects/team/secret").unwrap(), b"original");
}
