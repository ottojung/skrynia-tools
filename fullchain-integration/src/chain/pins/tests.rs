//! Non-vacuous RED/GREEN tests for runtime SHA256 pin enforcement.
use super::{PINS, hex, manifest, sha256_file, verify, verify_all};
use std::path::{Path, PathBuf};

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

#[test]
fn manifest_is_computed_from_bytes_not_filename() {
    let d = std::env::temp_dir().join(format!("fc-pin-dir-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    for (i, (name, _)) in PINS.iter().enumerate() {
        std::fs::write(d.join(name), format!("bytes-{i}")).unwrap();
    }
    let lines = manifest(&d).unwrap();
    assert_eq!(lines.len(), PINS.len());
    for (name, _) in PINS {
        let h = hex(&sha256_file(&Path::new(&d).join(name)).unwrap());
        assert!(lines.iter().any(|l| l == &format!("SHA256 {name} {h}")));
    }
    assert!(
        verify_all(&d).is_err(),
        "arbitrary bytes cannot match real pins"
    );
}
