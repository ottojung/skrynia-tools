use super::{fixtures, server_revoke};
use std::{fs, path::Path};
pub fn verify(path: &Path) {
    let file = path.to_str().unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let pending = serde_json::json!({"state":"revoking_grant","grant":saved["grant"]});
    fixtures::private(path, &serde_json::to_vec(&pending).unwrap());
    let denied = fixtures::command(&[
        "admin",
        "revoke-grant",
        "--credentials-file",
        file,
        "--url",
        "http://127.0.0.1:1",
        "--confirm-grant-id",
        "test123",
    ]);
    assert!(!denied.status.success());
    let (url, worker) = server_revoke::once("DELETE /grants/test123 ", 404);
    let output = fixtures::command(&[
        "admin",
        "revoke-grant",
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
    assert!(fs::read_to_string(path).unwrap().contains("grant_revoked"));
}
