//! Disposable loopback HTTP mock of the trusted edge (tests only).
use super::mock_io::{Seen, field, read_request};
use super::mock_reply::{respond, status};
use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub enum Reply {
    Ok(Vec<u8>),
    Status(u16),
    Oversized,
    Hang,
}

pub struct MockEdge {
    pub port: u16,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl MockEdge {
    pub fn start(reply: Reply, host: &str, origin: &str, cap: &str, count: usize) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let (host, origin, cap) = (host.to_owned(), origin.to_owned(), cap.to_owned());
        std::thread::spawn(move || {
            for stream in listener.incoming().take(count) {
                let mut stream = stream.unwrap();
                let request = read_request(&mut stream);
                let auth = field(&request, "x-skrynia-edge-capability") == cap;
                let route = request.request_line == "POST /_skrynia/ HTTP/1.1"
                    && field(&request, "host") == host
                    && field(&request, "origin") == origin
                    && field(&request, "content-type") == "application/octet-stream";
                let failure = status(auth, route);
                let bytes = respond(&reply, failure);
                let _ = stream.write_all(&bytes);
                sink.lock().unwrap().push(request);
            }
        });
        Self { port, seen }
    }

    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}
