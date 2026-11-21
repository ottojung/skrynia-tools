use super::{call, fixture, root_server};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn revoked_user_denied_for_store_ops_and_after_restart() {
    let (mut s, issuer, _) = root_server(true);
    let mut issue = call::req("POST", "/grants/root/users");
    issue
        .headers
        .insert("authorization".into(), format!("Bearer {issuer}"));
    issue.body = b"{}".to_vec();
    let v: serde_json::Value = serde_json::from_slice(&call::call(&s, &issue).body).unwrap();
    let (uid, tok) = (
        v["userId"].as_str().unwrap(),
        v["userToken"].as_str().unwrap(),
    );
    let mut del = call::req("DELETE", &format!("/grants/root/users/{uid}"));
    del.headers
        .insert("authorization".into(), format!("Bearer {issuer}"));
    assert_eq!(call::call(&s, &del).status, 200);
    let mut put = call::req("POST", "/store/team/x");
    put.headers
        .insert("authorization".into(), format!("Bearer {tok}"));
    put.body = b"data".to_vec();
    assert_eq!(call::call(&s, &put).status, 403);
    s.restart_with_guest(&fixture::guest());
    let mut put2 = call::req("POST", "/store/team/y");
    put2.headers
        .insert("authorization".into(), format!("Bearer {tok}"));
    put2.body = b"data".to_vec();
    assert_eq!(call::call(&s, &put2).status, 403);
}
