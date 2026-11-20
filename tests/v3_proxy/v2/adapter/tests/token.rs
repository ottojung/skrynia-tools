use super::{fixture::Gateway, kernel_fixture::Server};
#[test]
#[ignore = "requires real generic kernel, Rust Program0, v2 guest and gateway binary"]
fn private_operator_token_errors_fail_closed_without_stopping_gateway() {
    let guest = std::fs::read(std::env::var("SKRYNIA_V2_PROVIDER_WASM").unwrap()).unwrap();
    let kernel = Server::start(Some(&guest));
    kernel.seed_file("v2/objects/public/file.txt", b"abcdef");
    let gateway = Gateway::start(&kernel.url);
    let client = reqwest::blocking::Client::new();
    let path = format!("{}/store/public/file.txt", gateway.url);
    std::fs::write(&gateway.token_path, [0x99u8; 32]).unwrap();
    assert_eq!(client.get(&path).send().unwrap().status(), 502);
    std::fs::write(&gateway.token_path, [0x42u8; 32]).unwrap();
    assert_eq!(client.get(&path).send().unwrap().status(), 200);
    std::fs::remove_file(&gateway.token_path).unwrap();
    assert_eq!(client.get(&path).send().unwrap().status(), 503);
    std::fs::write(&gateway.token_path, [0x42u8; 32]).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&gateway.token_path, std::fs::Permissions::from_mode(0o644))
            .unwrap();
        assert_eq!(client.get(&path).send().unwrap().status(), 503);
        std::fs::set_permissions(&gateway.token_path, std::fs::Permissions::from_mode(0o600))
            .unwrap();
    }
    assert_eq!(client.get(&path).send().unwrap().status(), 200);
}
