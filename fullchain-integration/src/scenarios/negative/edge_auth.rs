use crate::{EdgeProc, Kernel};
pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let k = Kernel::start(art, &tmp.join("k")).expect("kernel");
    let e = EdgeProc::start(art, &tmp.join("e"), k.port, &k.token).expect("edge");
    let url = format!("http://127.0.0.1:{}/_skrynia/", e.port);
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let body = vec![1u8, 20]; // minimal frame, will be rejected before P1
    let bad_cap = client
        .post(&url)
        .header("Host", &e.host)
        .header("Origin", &e.origin)
        .header("X-Skrynia-Edge-Capability", "wrong-capability-0000000000")
        .header("Content-Type", "application/octet-stream")
        .body(body.clone())
        .send()
        .unwrap();
    let bad_host = client
        .post(&url)
        .header("Host", "evil.test")
        .header("Origin", &e.origin)
        .header("X-Skrynia-Edge-Capability", &e.cap)
        .header("Content-Type", "application/octet-stream")
        .body(body.clone())
        .send()
        .unwrap();
    let bad_origin = client
        .post(&url)
        .header("Host", &e.host)
        .header("Origin", "https://evil.test")
        .header("X-Skrynia-Edge-Capability", &e.cap)
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .send()
        .unwrap();
    assert_eq!(bad_cap.status().as_u16(), 401);
    assert_eq!(bad_host.status().as_u16(), 403);
    assert_eq!(bad_origin.status().as_u16(), 403);
    vec![
        "neg_edge_bad_cap=401".into(),
        "neg_edge_bad_host=403".into(),
        "neg_edge_bad_origin=403".into(),
    ]
}
