use super::{cases, fixture::Server, guest};

#[test]
#[ignore = "requires compiled Program 0 and exact generic-FD kernel"]
fn authenticated_compiled_proxy_streams_beyond_old_buffer_caps() {
    let s = Server::start(Some(&guest::echo()));
    let mut bytes = vec![0u8; 12 * 1024 * 1024 + 13];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = ((i * 37 + 17) & 255) as u8;
    }
    let body = cases::authenticated(&bytes, &[0x42; 32]);
    let response = cases::post(&s.url, &body);
    assert_eq!(response.status(), 200);
    assert_eq!(response.bytes().unwrap().as_ref(), bytes);
}
