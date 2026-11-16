use super::{fixtures, server_revoke};
use std::{fs, path::Path};
pub fn verify(path: &Path) {
    let file = path.to_str().unwrap();
    let (url, worker) = server_revoke::once("DELETE /grants/test123 ", 200);
    let output = fixtures::command(&[
        "admin",
        "revoke-grant",
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
    let saved = fs::read_to_string(path).unwrap();
    assert!(saved.contains("grant_revoked") && !saved.contains("test-private-issuer"));
}
