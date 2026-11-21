use super::call::{call, req};
use super::cases;
use super::fixture::Server;
#[test]
#[ignore = "needs compiled FD-WASM v2 guest, actual Rust Program0 and generic kernel"]
fn v2_provider_can_read_real_disk_via_generic_v3_callbacks() {
    let wasm = std::fs::read(std::env::var("SKRYNIA_V2_PROVIDER_WASM").unwrap()).unwrap();
    let s = Server::start(Some(&wasm));
    s.seed_file("v2/objects/public/file.txt", b"abcdef");
    let get = call(&s, &req("GET", "/store/public/file.txt"));
    assert_eq!(get.status, 200);
    assert_eq!(get.body, b"abcdef");
    let head = call(&s, &req("HEAD", "/store/public/file.txt"));
    assert_eq!(head.status, 200);
    assert!(head.body.is_empty());
    let mut ranged = req("GET", "/store/public/file.txt");
    ranged.headers.insert("range".into(), "bytes=2-3".into());
    let part = call(&s, &ranged);
    assert_eq!(part.status, 206);
    assert_eq!(part.body, b"cd");
    assert_eq!(
        call(&s, &req("GET", "/store/public/missing.txt")).status,
        404
    );
    assert_eq!(call(&s, &req("GET", "/store/../evil")).status, 400);
    // Admin routes are guest-owned and require a bearer; no kernel stub.
    assert_eq!(call(&s, &req("POST", "/grants")).status, 401);
    assert_eq!(call(&s, &req("PUT", "/store/public/file.txt")).status, 401);
    assert_eq!(
        reqwest::blocking::Client::new()
            .get(&s.url)
            .send()
            .unwrap()
            .status(),
        404
    );
    let raw = serde_json::to_vec(&req("GET", "/store/public/file.txt")).unwrap();
    assert_eq!(
        cases::post(&s.url, &cases::authenticated(&raw, &[0x99; 32])).status(),
        500
    );
}
