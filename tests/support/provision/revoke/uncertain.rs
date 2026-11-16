use super::{fixtures, server_revoke};
use std::fs;
pub fn verify() {
    let root = fixtures::root("revoke-unknown");
    let path = root.join("credentials.json");
    let file = path.to_str().unwrap();
    let grant = serde_json::json!({"id":"test123","issuerToken":"issuer"});
    let user = serde_json::json!({"userId":"user123","userToken":"user"});
    let record = serde_json::json!({"state":"revoking_user","grant":grant,"user":user});
    fixtures::private(&path, &serde_json::to_vec(&record).unwrap());
    let denied = fixtures::command(&[
        "admin",
        "revoke-user",
        "--credentials-file",
        file,
        "--url",
        "http://127.0.0.1:1",
        "--confirm-grant-id",
        "test123",
    ]);
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("retry-after-uncertain"));
    let (url, worker) = server_revoke::once("DELETE /grants/test123/users/user123 ", 404);
    let output = fixtures::command(&[
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
    assert!(output.status.success());
    worker.join().unwrap();
    assert!(fs::read_to_string(&path).unwrap().contains("user_revoked"));
    super::uncertain_grant::verify(&path);
    fs::remove_dir_all(root).unwrap();
}
