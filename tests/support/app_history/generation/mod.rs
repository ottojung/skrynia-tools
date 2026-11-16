use super::Fixture;
use std::fs;

pub fn verify(fixture: &Fixture, first: &serde_json::Value) {
    let root = fixture.apps.to_str().unwrap();
    let counter = fixture.apps.join(".locks/hello.generation");
    let original = fs::read(&counter).unwrap();
    assert_eq!(original.len(), 8);
    fs::write(&counter, b"broken").unwrap();
    let report = fixture.cli(&[
        "app",
        "audit",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--format",
        "json",
    ]);
    assert!(!report.status.success());
    checks::broken(&report);
    let failed = fixture.cli(&[
        "app",
        "rollback",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--release",
        first["id"].as_str().unwrap(),
    ]);
    assert!(
        !failed.status.success(),
        "corrupt generation allowed rollback"
    );
    fs::write(&counter, original).unwrap();
    let healthy = fixture.cli(&[
        "app",
        "audit",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--format",
        "json",
    ]);
    assert!(healthy.status.success(), "{:?}", healthy);
    checks::healthy(&healthy);
}

mod checks;
