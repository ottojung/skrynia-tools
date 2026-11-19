use super::*;
#[test]
fn grant_mutations_require_authority_and_well_formed_json() {
    let (b, _, root_user) = setup();
    assert_eq!(call(&b, "POST", "/grants", "", b"{}").0, 401);
    assert_eq!(
        call(&b, "POST", "/grants", &root_user, b"{").1["error"],
        "invalid_json"
    );
    let (status, error) = call(&b, "POST", "/grants", &root_user, &vec![b'x'; 8193]);
    assert_eq!(status, 400);
    assert_eq!(error["error"], "request_too_large");
    assert_eq!(call(&b, "POST", "/grants/r/users", "", b"").0, 403);
    assert_eq!(call(&b, "GET", "/grants/r", "", b"").0, 401);
}
