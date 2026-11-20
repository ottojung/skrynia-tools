use super::{fixture::Server, guest};
fn post(url: &str, body: &[u8]) -> reqwest::blocking::Response {
    reqwest::blocking::Client::new()
        .post(url)
        .header("content-type", "application/octet-stream")
        .body(body.to_vec())
        .send()
        .unwrap()
}
#[test]
#[ignore = "requires separately built root WASM and real v3 kernel"]
fn proxy_forwards_uninterpreted_bytes_to_only_operator_supplied_guest() {
    let s = Server::start(Some(&guest::echo()));
    let arbitrary = b"\0\xff\0POST\0GET\x01\x02\0";
    let response = post(&s.url, arbitrary);
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().unwrap().as_ref(), arbitrary);
}
#[test]
#[ignore = "requires separately built root WASM and real v3 kernel"]
fn missing_guest_fails_closed_without_fallback_protocol() {
    let s = Server::start(None);
    assert_eq!(post(&s.url, b"\x01\x01old-register").status(), 500);
}
#[test]
#[ignore = "requires separately built root WASM and real v3 kernel"]
fn unprivileged_guest_cannot_import_root_disk_syscalls() {
    let s = Server::start(Some(&guest::privileged()));
    assert_eq!(post(&s.url, b"anything").status(), 500);
}
