use super::*;
#[test]
fn recognized_mutations_route_as_not_implemented() {
    let c = handle(&Unavailable, &req("POST", "/store/a.txt"));
    assert_eq!(c.status, 401);
    assert_eq!(c.body, br#"{"error":"grant_user_token_required"}"#);
    let g = handle(&Unavailable, &req("POST", "/grants"));
    assert_eq!(g.status, 401);
    let mut authorized = req("POST", "/grants");
    authorized
        .headers
        .insert("Authorization".into(), format!("Bearer {}", "a".repeat(64)));
    authorized.body = br#"{}"#.to_vec();
    assert_eq!(handle(&Unavailable, &authorized).status, 501);
}
