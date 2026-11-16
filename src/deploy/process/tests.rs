use super::capture;
use std::process::Command;

#[test]
fn concurrent_capture_handles_pipe_capacity_without_deadlock() {
    let mut command = Command::new("sh");
    command.args(["-c", "head -c 100000 /dev/zero"]);
    let error = capture(command, 5).unwrap_err();
    assert!(error.to_string().contains("exceeds 64 KiB"), "{error}");
}
#[test]
fn capture_preserves_small_output() {
    let mut command = Command::new("sh");
    command.args(["-c", "printf 'hello world'"]);
    assert_eq!(capture(command, 5).unwrap(), b"hello world");
}
