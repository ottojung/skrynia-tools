use super::helpers::cli;
#[test]
fn invalid_options_and_required_arguments_fail_usefully() {
    let invalid = cli(&["app", "deploy", "--banana", "x"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("--banana"));
    let required = cli(&["app", "deploy", "--url", "http://localhost:1"]);
    assert_eq!(required.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&required.stderr).contains("--name"));
    let missing_url = cli(&["app", "list"]);
    assert_eq!(missing_url.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing_url.stderr).contains("--url"));
}
