mod stall;
use super::hold_client::send_and_hold_headers;
use super::{apps::two_apps, gateway::Gateway, http::post_raw, mock};
use std::io::Read;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

#[rustfmt::skip]
pub fn backpressure_nonreading_downstream() {
    let (up, stop, written) = mock::big_body_server(16 * 1024 * 1024);
    let pf = tempfile::NamedTempFile::new().unwrap().into_temp_path();
    let ps = pf.to_str().unwrap().to_string();
    let cap = "edge-cap-bp";
    let gw = Gateway::start_env(&format!("http://{up}/"), [0x42u8; 32], two_apps(cap, "edge-cap-b2"),
        serde_json::json!({"max_concurrent_requests":1,"request_timeout_ms":1500,"acquire_timeout_ms":250,"max_body_bytes":20*1024*1024}),
        &[("SKRYNIA_EDGE_GATEWAY_TEST_POLL_FILE", &ps)]);
    let mut c1 = send_and_hold_headers(gw.addr, "long.a.vau.place", "https://long.a.vau.place", cap, b"a");
    stall::wait_stall(&pf, &written);
    assert_eq!(req(gw.addr, cap).0, 503, "A: before deadline held permit must reject");
    thread::sleep(Duration::from_millis(2800));
    assert_eq!(req(gw.addr, cap).0, 200, "B: after deadline must recover");
    let _ = c1.set_read_timeout(Some(Duration::from_secs(2)));
    let mut tail = Vec::new(); let _ = c1.read_to_end(&mut tail);
    assert!(tail.len() < 16 * 1024 * 1024, "B: client1 must be terminated");
    drop(c1); thread::sleep(Duration::from_millis(100));
    assert_eq!(req(gw.addr, cap).0, 200, "C: explicit drop recovers");
    stop.store(true, Ordering::Relaxed);
}
#[rustfmt::skip]
fn req(addr: std::net::SocketAddr, cap: &str) -> (u16, Vec<(String,String)>, Vec<u8>) {
    post_raw(addr, "long.a.vau.place", Some("https://long.a.vau.place"), Some(cap), b"b")
}
