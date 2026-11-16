use super::SkryniaClient;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

#[test]
fn streamed_json_without_content_length_is_capped() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let writer = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut request = [0u8; 4096];
        let received = socket.read(&mut request).unwrap();
        assert!(received > 0);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n")
            .unwrap();
        socket.write_all(&vec![b'x'; 1_048_577]).ok();
    });
    let client = SkryniaClient::new(&url).unwrap().with_token("token".into());
    let error = client.list_page("big.*", None).unwrap_err();
    assert!(error.to_string().contains("exceeds 1 MiB"), "{error}");
    writer.join().unwrap();
}
