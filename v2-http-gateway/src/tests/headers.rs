use super::super::headers;
use skrynia_tools::wire::v2compat::Response;
use std::collections::BTreeMap;
#[test]
fn strips_hop_by_hop_response_headers_including_connection_nominations() {
    let mut fields = BTreeMap::new();
    fields.insert("Connection".into(), "X-Secret, Keep-Alive".into());
    fields.insert("X-Secret".into(), "private".into());
    fields.insert("Proxy-Authenticate".into(), "basic realm=hidden".into());
    fields.insert("Transfer-Encoding".into(), "chunked".into());
    fields.insert("X-Visible".into(), "safe".into());
    let response = headers::into_response(Response {
        status: 200,
        headers: fields,
        body: b"public".to_vec(),
    })
    .unwrap();
    assert!(response.headers().get("connection").is_none());
    assert!(response.headers().get("x-secret").is_none());
    assert!(response.headers().get("proxy-authenticate").is_none());
    assert!(response.headers().get("transfer-encoding").is_none());
    assert_eq!(response.headers().get("x-visible").unwrap(), "safe");
}
#[test]
fn strips_connection_nominated_and_proxy_credentials_from_requests() {
    let mut request = axum::http::HeaderMap::new();
    request.insert("connection", "x-secret".parse().unwrap());
    request.insert("x-secret", "private".parse().unwrap());
    request.insert("proxy-authorization", "Basic leaked".parse().unwrap());
    request.insert("authorization", "Bearer user-token".parse().unwrap());
    let filtered = headers::from_request(&request).unwrap();
    assert!(!filtered.contains_key("x-secret"));
    assert!(!filtered.contains_key("proxy-authorization"));
    assert!(!filtered.contains_key("connection"));
    assert_eq!(filtered.get("authorization").unwrap(), "Bearer user-token");
}
