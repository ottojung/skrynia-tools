use super::*;
#[test]
fn unknown_and_method_errors() {
    let n = handle(&Unavailable, &req("GET", "/nope"));
    assert_eq!(n.status, 404);
    let m = handle(&Unavailable, &req("POST", "/health"));
    assert_eq!(m.status, 405);
    assert_eq!(handle(&Unavailable, &req("GET", "/grants")).status, 404);
    assert_eq!(
        handle(&Unavailable, &req("GET", "/grants/r/users")).status,
        404
    );
    assert_eq!(handle(&Unavailable, &req("POST", "/grants/r")).status, 404);
    let k = handle(&Unavailable, &req("GET", "/store/bad%2"));
    assert_eq!(k.status, 400);
    assert_eq!(k.body, br#"{"error":"invalid_percent_encoding"}"#);
}
