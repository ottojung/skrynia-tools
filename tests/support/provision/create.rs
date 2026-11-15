use super::{fixtures, server_drop};
use std::fs;
pub fn verify() {
    let root = fixtures::root("lost-user-reply");
    let token = root.join("parent");
    fixtures::private(&token, b"parent-user-token");
    let file = root.join("creds.json");
    let (url, worker) = server_drop::create_then_drop();
    let output = fixtures::command(&[
        "admin",
        "create",
        "--name",
        "app",
        "--url",
        &url,
        "--parent-token-file",
        token.to_str().unwrap(),
        "--credentials-out",
        file.to_str().unwrap(),
        "--max-bytes",
        "10000",
        "--max-objects",
        "10",
    ]);
    worker.join().unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("issuer-top-secret"));
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(saved["state"], "issuing_user");
    assert_eq!(saved["grant"]["issuerToken"], "issuer-top-secret");
    let inspect = fixtures::command(&[
        "admin",
        "inspect",
        "--credentials-file",
        file.to_str().unwrap(),
    ]);
    assert!(inspect.status.success());
    assert!(!String::from_utf8_lossy(&inspect.stdout).contains("issuer-top-secret"));
    fs::remove_dir_all(root).unwrap();
}
