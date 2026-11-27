mod support;

#[test]
fn slow_unfinished_chunked_recovers() {
    support::liveness::slow_unfinished_chunked_recovers();
}

#[test]
fn silent_upstream_times_out_and_recovers() {
    support::liveness::silent_upstream_times_out_and_recovers();
}

#[test]
fn saturated_slots_then_benign_responsive() {
    support::liveness::saturated_slots_then_benign_responsive();
}

#[test]
fn stderr_drain_captures_child_diagnostics() {
    use std::process::{Command, Stdio};
    let mut child = Command::new("sh")
        .args(["-c", "printf EDGE-DIAG-MARKER >&2"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut drain = support::kit::spawn_stderr_drain(&mut child);
    let _ = child.wait();
    drain.finish();
    assert!(drain.snapshot().contains("EDGE-DIAG-MARKER"));
}

#[test]
#[should_panic(expected = "EDGE-EARLY-EXIT-MARKER")]
fn wait_ready_reports_child_stderr_on_early_exit() {
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;
    let mut child = Command::new("sh")
        .args(["-c", "printf EDGE-EARLY-EXIT-MARKER >&2; exit 3"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let drain = support::kit::spawn_stderr_drain(&mut child);
    thread::sleep(Duration::from_millis(100));
    let addr = format!("127.0.0.1:{}", support::kit::free_port())
        .parse()
        .unwrap();
    support::kit::wait_ready(addr, &mut child, &drain.buf);
}
