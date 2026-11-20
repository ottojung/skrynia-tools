use super::{Fake, req};
use skrynia_tools::wire::v2compat::{Request, Response};
use skrynia_v2_provider::dispatch_bytes;
#[test]
fn legacy_http_method_is_opaque_v3_post_payload_data() {
    let request = req("GET", "/store/public/file.txt");
    let input = serde_json::to_vec(&request).unwrap();
    let response: Response =
        serde_json::from_slice(&dispatch_bytes(&input, &Fake).unwrap()).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"abcdef");
    let back: Request = serde_json::from_slice(&input).unwrap();
    assert_eq!(back.method, "GET");
}
#[test]
fn malformed_and_oversized_frames_do_not_invoke_any_backend() {
    assert!(dispatch_bytes(b"{not-json", &Fake).is_err());
    assert!(dispatch_bytes(&vec![b'a'; 8 * 1024 * 1024 + 1], &Fake).is_err());
}

#[test]
fn unknown_request_envelope_fields_are_denied() {
    let raw = br#"{"method":"GET","path":"/health","headers":{},
      "query":null,"body":[],"operator_token":"leak"}"#;
    assert!(dispatch_bytes(raw, &Fake).is_err());
}
