//! Ephemeral loopback port allocation for disposable tests.
use std::net::TcpListener;

pub fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind");
    l.local_addr().expect("addr").port()
}
