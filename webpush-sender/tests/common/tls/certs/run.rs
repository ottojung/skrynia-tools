//! Silent openssl invocation; failures panic with the failing argument list.
use std::process::Command;

pub fn run(args: Vec<&str>) {
    let status = Command::new("openssl")
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "openssl failed: {}", args.join(" "));
}
