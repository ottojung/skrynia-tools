use std::process::Command;

#[test]
fn bounded_status_probe_preserves_exit_code() {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "exit 7"]);
    assert_eq!(super::status(cmd, 2).unwrap().code(), Some(7));
}
#[test]
fn bounded_status_probe_times_out() {
    let mut cmd = Command::new("sh");
    cmd.args(["-c", "sleep 30"]);
    let error = super::status(cmd, 1).unwrap_err();
    assert!(error.to_string().contains("timed out"), "{error}");
}
