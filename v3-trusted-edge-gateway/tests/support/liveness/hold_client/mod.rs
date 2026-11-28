mod connect;
mod prefix;
mod sockopt;

use std::net::{SocketAddr, TcpStream};

use self::connect::send_and_hold;

pub fn send_and_hold_headers(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    cap: &str,
    body: &[u8],
) -> TcpStream {
    send_and_hold(addr, host, origin, cap, body, false).0
}

/// Scoped to the backpressure proof: constrain the receive buffer and return
/// the already-read response prefix so the caller can prove it stayed partial.
pub fn send_and_hold_headers_small_rcvbuf(
    addr: SocketAddr,
    host: &str,
    origin: &str,
    cap: &str,
    body: &[u8],
) -> (TcpStream, Vec<u8>) {
    send_and_hold(addr, host, origin, cap, body, true)
}
