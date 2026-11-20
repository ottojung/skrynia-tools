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
