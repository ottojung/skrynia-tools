use super::*;
#[test]
fn issuer_revocation_respects_mutation_authority() {
    let (b, _, root_user) = setup();
    let new = child(&b, &root_user);
    let id = new["id"].as_str().unwrap();
    let issuer = new["issuerToken"].as_str().unwrap();
    let (status, user) = call(&b, "POST", &format!("/grants/{id}/users"), issuer, b"");
    assert_eq!(status, 200);
    let token = user["userToken"].as_str().unwrap();
    let user_id = user["userId"].as_str().unwrap();
    let (status, removed) = call(
        &b,
        "DELETE",
        &format!("/grants/{id}/users/{user_id}"),
        issuer,
        b"",
    );
    assert_eq!(status, 200);
    assert_eq!(removed["ok"], true);
    assert_eq!(call(&b, "GET", &format!("/grants/{id}"), token, b"").0, 403);
    assert_eq!(
        call(&b, "DELETE", &format!("/grants/{id}"), issuer, b"").0,
        200
    );
    assert_eq!(
        call(&b, "POST", &format!("/grants/{id}/users"), issuer, b"").0,
        404
    );
}
