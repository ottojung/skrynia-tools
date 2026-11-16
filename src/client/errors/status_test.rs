use super::{ClientError, SkryniaClient, Store, tests::response};
use reqwest::StatusCode;

#[test]
fn accepted_create_is_not_a_committed_create() {
    let reply = "HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let client = SkryniaClient::new(&response(Some(reply)))
        .unwrap()
        .with_token("token".into());
    let error = client.create("example", b"hello").unwrap_err();
    assert!(matches!(error.downcast_ref::<ClientError>(),
        Some(ClientError::UnexpectedMutationStatus { actual, expected, .. })
        if *actual == StatusCode::ACCEPTED && *expected == StatusCode::CREATED));
}
#[test]
fn no_content_put_is_not_a_confirmed_mutation() {
    let reply = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let client = SkryniaClient::new(&response(Some(reply)))
        .unwrap()
        .with_token("token".into());
    let error = client
        .compare_replace("example", b"data", &"0".repeat(64))
        .unwrap_err();
    assert!(matches!(error.downcast_ref::<ClientError>(),
        Some(ClientError::UnexpectedMutationStatus { actual, expected, .. })
        if *actual == StatusCode::NO_CONTENT && *expected == StatusCode::OK));
}

#[test]
fn partial_get_is_not_a_complete_object() {
    let reply =
        "HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\nConnection: close\r\n\r\npart";
    let client = SkryniaClient::new(&response(Some(reply))).unwrap();
    let error = client.read("example").unwrap_err();
    assert!(matches!(error.downcast_ref::<ClientError>(),
        Some(ClientError::UnexpectedReadStatus { actual, expected, .. })
        if *actual == StatusCode::PARTIAL_CONTENT && *expected == StatusCode::OK));
}

#[test]
fn oversized_list_metadata_is_rejected_without_buffering_entire_response() {
    let reply = "HTTP/1.1 200 OK\r\nContent-Length: 2000000\r\nConnection: close\r\n\r\n";
    let client = SkryniaClient::new(&response(Some(reply)))
        .unwrap()
        .with_token("token".into());
    let error = client.list_page("test.*", None).unwrap_err();
    assert!(error.to_string().contains("exceeds 1 MiB"), "{error}");
}
