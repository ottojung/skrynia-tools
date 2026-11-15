use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
pub fn once() -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut request = [0u8; 4096];
        let used = socket.read(&mut request).unwrap();
        assert!(
            String::from_utf8_lossy(&request[..used]).starts_with("POST /grants/test123/users ")
        );
        let body = r#"{"userToken":"issued-very-secret-user"}"#;
        let response = format!(
            "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).unwrap();
    });
    (url, handle)
}
