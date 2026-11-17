use super::helpers::cli;
#[test]
fn version_and_help_have_standard_cli_behavior() {
    let version = cli(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        format!("skrynia-tools {}", env!("CARGO_PKG_VERSION"))
    );
    let help = cli(&["--help"]);
    assert!(help.status.success());
    let body = String::from_utf8_lossy(&help.stdout);
    assert!(body.contains("Manage applications, objects, logs, and grants"));
    assert!(body.contains("--version"));
    assert!(!body.contains("HTTP-only"));
    assert!(!body.contains("builds on this machine"));
    let grant = cli(&["admin", "grant", "--help"]);
    assert!(grant.status.success());
    let body = String::from_utf8_lossy(&grant.stdout);
    assert!(body.contains("--parent-grant-is-root"));
    assert!(body.contains("--parent-grant"));
    assert!(cli(&["admin", "grant", "--version"]).status.success());
}
