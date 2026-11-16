use super::transport;
use std::{
    process::Command,
    time::{Duration, Instant},
};
#[test]
fn blocked_stdin_writer_cannot_bypass_deadline() {
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 15"]);
    let start = Instant::now();
    let error = transport::send(command, &vec![b'a'; 1024 * 1024], 1).unwrap_err();
    assert!(error.to_string().contains("timed out"), "{error}");
    assert!(start.elapsed() < Duration::from_secs(5));
}
#[test]
fn successful_send_requires_consumption_and_exit_success() {
    let mut command = Command::new("sh");
    command.args(["-c", "cat >/dev/null"]);
    transport::send(command, b"hello", 3).unwrap();
    let mut bad = Command::new("sh");
    bad.args(["-c", "cat >/dev/null; exit 7"]);
    assert!(transport::send(bad, b"payload", 3).is_err());
}
