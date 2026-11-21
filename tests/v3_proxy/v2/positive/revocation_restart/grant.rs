use super::{call, fixture, root_server};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn revoked_child_grant_denied_for_store_ops_after_restart() {
    let (mut s, _, user) = root_server(true);
    let mut req = call::req("POST", "/grants");
    req.headers
        .insert("authorization".into(), format!("Bearer {user}"));
    req.body =
        br#"{"locations":"team/c/*","operations":["create"],"maxBytes":4096,"maxObjects":2}"#
            .to_vec();
    let v: serde_json::Value = serde_json::from_slice(&call::call(&s, &req).body).unwrap();
    let (id, child_iss) = (
        v["id"].as_str().unwrap(),
        v["issuerToken"].as_str().unwrap(),
    );
    let mut issue = call::req("POST", &format!("/grants/{id}/users"));
    issue
        .headers
        .insert("authorization".into(), format!("Bearer {child_iss}"));
    issue.body = b"{}".to_vec();
    let tok = serde_json::from_slice::<serde_json::Value>(&call::call(&s, &issue).body).unwrap()
        ["userToken"]
        .as_str()
        .unwrap()
        .to_string();
    let mut del = call::req("DELETE", &format!("/grants/{id}"));
    del.headers
        .insert("authorization".into(), format!("Bearer {child_iss}"));
    assert_eq!(call::call(&s, &del).status, 200);
    let mut put = call::req("POST", "/store/team/c/a");
    put.headers
        .insert("authorization".into(), format!("Bearer {tok}"));
    put.body = b"data".to_vec();
    assert_eq!(call::call(&s, &put).status, 403);
    s.restart_with_guest(&fixture::guest());
    let mut put2 = call::req("POST", "/store/team/c/b");
    put2.headers
        .insert("authorization".into(), format!("Bearer {tok}"));
    put2.body = b"data".to_vec();
    assert_eq!(call::call(&s, &put2).status, 403);
}
