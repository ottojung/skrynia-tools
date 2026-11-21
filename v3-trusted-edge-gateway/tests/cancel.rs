mod support;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use support::{apps::two_apps, gateway::Gateway, http::post_raw, kernel::Kernel, kit::kit};

#[test]
fn cancellation_and_recovery() {
    let Some((bin, p0)) = kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let token = [0x42u8; 32];
    let cap = "edge-cap-cancel";
    let kernel = Kernel::start(&bin, &p0, token);
    let gw = Gateway::start(
        &kernel.url(),
        token,
        two_apps(cap, "edge-cap-other"),
        16 * 1024 * 1024,
    );
    let mut stream = TcpStream::connect(gw.addr).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let body_len = 8 * 1024 * 1024;
    let req = format!(
        "POST /_skrynia/ HTTP/1.1\r\nHost: long.a.vau.place\r\nOrigin: https://long.a.vau.place\r\nX-Skrynia-Edge-Capability: {cap}\r\nContent-Type: application/octet-stream\r\nContent-Length: {body_len}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).unwrap();
    let _ = stream.write_all(&vec![7u8; 64 * 1024]);
    drop(stream);
    std::thread::sleep(Duration::from_millis(200));
    let (st, _, body) = post_raw(
        gw.addr,
        "long.a.vau.place",
        Some("https://long.a.vau.place"),
        Some(cap),
        b"after-cancel",
    );
    assert_eq!(st, 200);
    assert_eq!(body, b"after-cancel");
}
