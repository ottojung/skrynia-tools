mod client;
mod gateway;
mod stall;

use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use super::hold_client::send_and_hold_headers_small_rcvbuf;
use super::{
    apps::two_apps,
    gateway::Gateway,
    http::{dechunk, post_raw},
    mock,
};

const TOTAL: usize = 16 * 1024 * 1024;
const OPERATOR: [u8; 32] = [0x42u8; 32];
const REQUEST_TIMEOUT_MS: u64 = 2000;

pub fn backpressure_nonreading_downstream() {
    let (up, stop, written) = mock::big_body_server(TOTAL, OPERATOR);
    let cap = "edge-cap-bp";
    let gw = gateway::start(up, cap);
    let deadline = Instant::now() + Duration::from_millis(REQUEST_TIMEOUT_MS);
    let (mut c1, prefix) = send_and_hold_headers_small_rcvbuf(
        gw.addr,
        "long.a.vau.place",
        "https://long.a.vau.place",
        cap,
        b"a",
    );
    stall::wait_backpressure(&written, TOTAL, deadline);

    assert_eq!(
        gateway::request(gw.addr, cap).0,
        503,
        "A: before deadline held permit must reject"
    );
    thread::sleep(Duration::from_millis(2800));
    gateway::assert_recovery(gw.addr, cap, "B: after deadline must recover");
    client::assert_terminated(&mut c1, prefix);

    drop(c1);
    thread::sleep(Duration::from_millis(100));
    gateway::assert_recovery(gw.addr, cap, "C: explicit drop recovers");
    stop.store(true, Ordering::Relaxed);
}
