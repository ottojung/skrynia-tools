use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
pub fn once(expected: &str, code: u16) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    respond(listener, expected.to_owned(), code, None, url)
}
pub fn unexpected_404(expected: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    respond(
        listener,
        expected.into(),
        404,
        Some(r#"{"error":"unrelated"}"#),
        url,
    )
}
fn respond(
    listener: TcpListener,
    expected: String,
    code: u16,
    custom: Option<&'static str>,
    url: String,
) -> (String, thread::JoinHandle<()>) {
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0u8; 4096];
        let read = stream.read(&mut request).unwrap();
        assert!(String::from_utf8_lossy(&request[..read]).starts_with(&expected));
        let body = if let Some(value) = custom {
            value
        } else if code == 404 && expected.contains("/users/") {
            r#"{"error":"user_not_found"}"#
        } else if code == 404 {
            r#"{"error":"grant_not_found"}"#
        } else {
            "{}"
        };
        let response = format!(
            "HTTP/1.1 {code} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, handle)
}
