use super::{fixtures, server};
use std::fs;

pub fn verify() {
    let root = fixtures::root("ambiguous");
    let file = root.join("creds.json");
    let path = file.to_str().unwrap();
    let record = serde_json::json!({"state":"issuing_user",
        "grant":{"id":"test123","issuerToken":"issuer-top-secret"}});
    fixtures::private(&file, &serde_json::to_vec(&record).unwrap());
    let denied = fixtures::command(&[
        "admin",
        "resume",
        "--credentials-file",
        path,
        "--url",
        "http://127.0.0.1:1",
    ]);
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("accept-duplicate-token-risk"));
    let (url, worker) = server::once();
    let accepted = fixtures::command(&[
        "admin",
        "resume",
        "--credentials-file",
        path,
        "--url",
        &url,
        "--accept-duplicate-token-risk",
        "true",
    ]);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    worker.join().unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(updated["user"]["userToken"], "issued-very-secret-user");
    fs::remove_dir_all(root).unwrap();
}
