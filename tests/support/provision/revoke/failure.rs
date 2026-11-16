use super::{fixtures, server_revoke};
use std::fs;

pub fn verify() {
    let root = fixtures::root("revoke-failure");
    let path = root.join("credentials.json");
    let file = path.to_str().unwrap();
    let record = serde_json::json!({"grant":{"id":"test123","issuerToken":"issuer-secret"},
        "user":{"userId":"user123","userToken":"user-secret"}});
    fixtures::private(&path, &serde_json::to_vec(&record).unwrap());
    let (url, worker) = server_revoke::once("DELETE /grants/test123/users/user123 ", 503);
    let output = fixtures::command(&[
        "admin",
        "revoke-user",
        "--credentials-file",
        file,
        "--url",
        &url,
        "--confirm-grant-id",
        "test123",
    ]);
    worker.join().unwrap();
    assert!(!output.status.success());
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.contains("revoking_user") && saved.contains("issuer-secret"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("issuer-secret"));
    let (url, worker) = server_revoke::unexpected_404("DELETE /grants/test123/users/user123 ");
    let retry = fixtures::command(&[
        "admin",
        "revoke-user",
        "--credentials-file",
        file,
        "--url",
        &url,
        "--confirm-grant-id",
        "test123",
        "--retry-after-uncertain",
        "true",
    ]);
    worker.join().unwrap();
    assert!(!retry.status.success());
    assert!(fs::read_to_string(&path).unwrap().contains("revoking_user"));
    fs::remove_dir_all(root).unwrap();
}
