//! Non-vacuous RED/GREEN tests for runtime SHA256 pin enforcement.
use super::{hex, sha256_file, verify};
use std::path::PathBuf;

fn tmp(label: &str, bytes: &[u8]) -> PathBuf {
    let p = std::env::temp_dir().join(format!("fc-pin-{label}-{}", std::process::id()));
    std::fs::write(&p, bytes).unwrap();
    p
}

#[test]
fn green_correct_pin_accepted() {
    let p = tmp("green", b"pinned-bytes");
    let h = hex(&sha256_file(&p).unwrap());
    verify(&p, &h).expect("correct runtime hash must be accepted");
}

#[test]
fn red_wrong_pin_rejected() {
    let p = tmp("wrong", b"pinned-bytes");
    let bad = hex(&[0u8; 32]);
    assert!(verify(&p, &bad).is_err(), "wrong pin must fail closed");
}

#[test]
fn red_tampered_artifact_rejected() {
    let p = tmp("tamper", b"original-bytes");
    let h = hex(&sha256_file(&p).unwrap());
    std::fs::write(&p, b"tampered-bytes").unwrap();
    assert!(verify(&p, &h).is_err(), "tampered bytes must fail closed");
}
