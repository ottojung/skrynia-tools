use super::helpers::cli;
#[test]
fn mutually_exclusive_parent_sources_reject_ambiguity() {
    let bad = cli(&[
        "admin",
        "grant",
        "--url",
        "http://localhost:1",
        "--parent-grant-is-root",
        "--parent-grant",
        "f".repeat(64).as_str(),
    ]);
    assert_eq!(bad.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad.stderr).contains("cannot be used"));
}
