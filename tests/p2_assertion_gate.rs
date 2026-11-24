//! CI gate for the Theorem Lab P2 security-assertion non-vacuity check.
//!
//! The existing `cargo test --locked --all-targets` CI steps run this test,
//! which invokes a stdlib-only Python script. No kernel, WASM, credentials or
//! provider traffic are involved; only the assertion predicates are checked.
use std::process::Command;

#[test]
fn theorem_p2_security_assertions_are_non_vacuous() {
    let out = Command::new("python3")
        .arg("examples/theorem_lab_feedback_p2/tests/nonvacuity.py")
        .output()
        .expect("python3 is required for the P2 assertion non-vacuity gate");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "P2 assertion non-vacuity gate failed: {stdout}{stderr}"
    );
    assert!(
        stdout.contains("ALL_OK=True"),
        "P2 assertion non-vacuity gate did not report ALL_OK: {stdout}"
    );
    assert!(
        stdout.contains("old-forge-check-was-vacuous")
            && stdout.contains("old-sibling-check-was-vacuous"),
        "RED-on-old demonstrations missing: {stdout}"
    );
}
