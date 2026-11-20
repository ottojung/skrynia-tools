use super::{fixture::Gateway, kernel_fixture::Server};
#[test]
#[ignore = "requires real generic kernel, Rust Program0, v2 guest and gateway binary"]
fn legacy_http_gateway_preserves_v2_without_disclosing_operator_token() {
    let guest = std::fs::read(std::env::var("SKRYNIA_V2_PROVIDER_WASM").unwrap()).unwrap();
    let kernel = Server::start(Some(&guest));
    kernel.seed_file("v2/objects/public/file.txt", b"abcdef");
    let gateway = Gateway::start(&kernel.url);
    let client = reqwest::blocking::Client::new();
    let path = format!("{}/store/public/file.txt", gateway.url);
    let get = client.get(&path).send().unwrap();
    assert_eq!(get.status(), 200);
    assert_eq!(get.bytes().unwrap().as_ref(), b"abcdef");
    let head = client.head(&path).send().unwrap();
    assert_eq!(head.status(), 200);
    assert_eq!(head.bytes().unwrap().len(), 0);
    let part = client
        .get(&path)
        .header("Range", "bytes=2-3")
        .send()
        .unwrap();
    assert_eq!(part.status(), 206);
    assert_eq!(part.bytes().unwrap().as_ref(), b"cd");
    let preflight = client
        .request(reqwest::Method::OPTIONS, &path)
        .header("Origin", "https://example.org")
        .header("Access-Control-Request-Method", "PUT")
        .send()
        .unwrap();
    assert_eq!(preflight.status(), 204);
    assert_eq!(
        preflight
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );
    let put = client.put(&path).body("new bytes").send().unwrap();
    assert_eq!(put.status(), 401);
    assert_eq!(
        kernel.read_file("v2/objects/public/file.txt").unwrap(),
        b"abcdef"
    );
    assert_eq!(client.get(&kernel.url).send().unwrap().status(), 404);
}
