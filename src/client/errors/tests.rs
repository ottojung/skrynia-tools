use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn response(reply: Option<&str>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let message = reply.map(str::to_owned);
    std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 4096];
        let _ = socket.read(&mut buffer);
        if let Some(value) = message {
            socket.write_all(value.as_bytes()).unwrap();
        }
    });
    format!("http://{address}")
}
#[test]
fn preserves_quota_error_and_status() {
    let body = r#"{"error":"grant_quota_exceeded"}"#;
    let reply = format!(
        "HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let client = SkryniaClient::new(&response(Some(&reply)))
        .unwrap()
        .with_token("token".to_string());
    let error = client.create("example", b"hello").unwrap_err();
    assert!(matches!(error.downcast_ref::<ClientError>(),
        Some(ClientError::Rejected { status, code })
        if *status == StatusCode::FORBIDDEN && code == "grant_quota_exceeded"));
}
#[test]
fn lost_post_reply_is_explicitly_unknown() {
    let client = SkryniaClient::new(&response(None))
        .unwrap()
        .with_token("token".into());
    let error = client.create("example", b"hello").unwrap_err();
    assert!(matches!(
        error.downcast_ref::<ClientError>(),
        Some(ClientError::OutcomeUnknown { .. })
    ));
}
