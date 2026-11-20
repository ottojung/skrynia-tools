use super::{fixture::Server, guest};

#[test]
#[ignore = "requires independently built streaming kernel and Program0"]
fn ten_mib_echo_crosses_old_eight_mib_buffer_limit() {
    let server = Server::start(Some(&guest::echo()));
    let bytes: Vec<u8> = (0usize..10 * 1024 * 1024)
        .map(|n| (n.wrapping_mul(73) % 251) as u8)
        .collect();
    let response = reqwest::blocking::Client::new()
        .post(&server.url)
        .header("content-type", "application/octet-stream")
        .body(bytes.clone())
        .send()
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().unwrap().as_ref(), bytes);
    let next = reqwest::blocking::Client::new()
        .post(&server.url)
        .body(b"another-request".to_vec())
        .send()
        .unwrap();
    assert_eq!(next.status(), 200);
    assert_eq!(next.bytes().unwrap(), b"another-request".as_slice());
}
