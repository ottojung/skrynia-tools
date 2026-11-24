mod support;
use std::{path::Path, process::Command};

#[test]
fn compiled_service_roundtrip() {
    let Some((kernel, p0)) = support::kit::kit() else {
        eprintln!("SKIP: canonical kit required");
        return;
    };
    let p1 = std::env::var("SKRYNIA_V3_NATIVE_SERVICE_WASM")
        .expect("ordinary service artifact required");
    assert!(Path::new(&p1).is_file());
    let work = tempfile::tempdir().unwrap();
    let echo = work.path().join("echo.wasm");
    std::fs::write(&echo, support::kernel::wasm::echo_wasm()).unwrap();
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../program0/integration/dumb_p0_http.py");
    let status = Command::new("python3")
        .arg(script)
        .args(["--kernel", &kernel, "--p0", &p0, "--p1", &p1])
        .arg("--echo-guest")
        .arg(echo)
        .status()
        .unwrap();
    assert!(status.success(), "compiled ordinary service failed");
}
