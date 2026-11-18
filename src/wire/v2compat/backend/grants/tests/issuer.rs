use super::*;
#[test]
fn v2_issuer_is_not_user_token() {
    let auth = authority();
    assert!(auth.issuer("root", "issuer").is_ok());
    assert_eq!(
        auth.user("issuer").unwrap_err().code,
        "invalid_grant_user_token"
    );
    assert_eq!(
        auth.issuer("root", "token").unwrap_err().code,
        "invalid_grant_issuer_token"
    );
    assert_eq!(
        auth.issuer("absent", "issuer").unwrap_err().code,
        "grant_not_found"
    );
}
