use super::{fixtures, server_revoke};
use std::fs;
pub fn verify() {
    let root = fixtures::root("revoke-success");
    let path = root.join("credentials.json");
    let record = serde_json::json!({"grant":{"id":"test123","issuerToken":"test-private-issuer"},
        "user":{"userId":"user123","userToken":"test-private-user"}});
    fixtures::private(&path, &serde_json::to_vec(&record).unwrap());
    let file = path.to_str().unwrap();
    let rejected = fixtures::command(&[
        "admin",
        "revoke-user",
        "--credentials-file",
        file,
        "--url",
        "http://127.0.0.1:1",
        "--confirm-grant-id",
        "wrong",
    ]);
    assert!(!rejected.status.success());
    let (url, worker) = server_revoke::once("DELETE /grants/test123/users/user123 ", 200);
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
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    worker.join().unwrap();
    let saved = fs::read_to_string(&path).unwrap();
    assert!(saved.contains("user_revoked") && !saved.contains("test-private-user"));
    super::complete::verify(&path);
    fs::remove_dir_all(root).unwrap();
}
