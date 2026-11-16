use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
pub fn create_then_drop() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        for step in 0..2 {
            let (mut socket, _) = listener.accept().unwrap();
            let mut req = [0u8; 8192];
            let read = socket.read(&mut req).unwrap();
            let request = String::from_utf8_lossy(&req[..read]);
            if step == 0 {
                assert!(request.starts_with("POST /grants HTTP/1.1"));
                let body = r#"{"id":"test123","issuerToken":"issuer-top-secret"}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(response.as_bytes()).unwrap();
            } else {
                assert!(request.starts_with("POST /grants/test123/users HTTP/1.1"));
                // The request may have committed despite a lost response.
            }
        }
    });
    (url, handle)
}
