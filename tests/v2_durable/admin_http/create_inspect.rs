use super::*;
#[test]
fn delegated_grant_issue_and_inspection_over_http() {
    let (b, _, root_user) = setup();
    let new = child(&b, &root_user);
    let id = new["id"].as_str().unwrap();
    assert_eq!(id.len(), 32);
    let issuer = new["issuerToken"].as_str().unwrap();
    assert_eq!(issuer.len(), 64);
    let (status, user) = call(&b, "POST", &format!("/grants/{id}/users"), issuer, b"");
    assert_eq!(status, 200);
    assert_eq!(user["userId"].as_str().unwrap().len(), 32);
    let token = user["userToken"].as_str().unwrap();
    let (status, inspected) = call(&b, "GET", &format!("/grants/{id}"), token, b"");
    assert_eq!(status, 200);
    assert_eq!(inspected["userCount"], 1);
    assert_eq!(inspected["usage"]["bytes"], 2048);
    assert_eq!(inspected["usage"]["objects"], 0);
    assert!(inspected.get("issuerHash").is_none());
    assert!(inspected.get("users").is_none());
    assert_eq!(
        call(&b, "GET", &format!("/grants/{id}"), &root_user, b"").0,
        403
    );
}
