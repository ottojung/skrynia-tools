use super::{capture, run};
use std::process::Command;

#[test]
fn timed_out_build_kills_child_process_group() {
    use std::{fs, time::Duration};
    let output =
        std::env::temp_dir().join(format!("skrynia-process-{:032x}", rand::random::<u128>()));
    let mut command = Command::new("sh");
    let script = format!("(sleep 2; printf leaked > {}) & wait", output.display());
    command.args(["-c", &script]);
    let error = run(command, 1).unwrap_err();
    assert!(error.to_string().contains("timed out"), "{error}");
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        !output.exists(),
        "timed-out descendant escaped its process group"
    );
    let _ = fs::remove_file(output);
}

#[test]
fn abandoned_stdout_pipe_does_not_leave_background_writer() {
    use std::{fs, time::Duration};
    let output =
        std::env::temp_dir().join(format!("skrynia-reader-{:032x}", rand::random::<u128>()));
    let mut command = Command::new("sh");
    let script = format!("(sleep 2; printf leaked > {}) & exit 0", output.display());
    command.args(["-c", &script]);
    let error = capture(command, 1).unwrap_err();
    assert!(
        error.to_string().contains("stdout remained open"),
        "{error}"
    );
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        !output.exists(),
        "background writer survived capture timeout"
    );
    let _ = fs::remove_file(output);
}
