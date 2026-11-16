use super::Fixture;
mod cases;
mod metadata;
mod missing;
mod tamper;

pub fn verify(fixture: &Fixture, first: &serde_json::Value) {
    let root = fixture.apps.to_str().unwrap();
    let healthy = fixture.cli(&[
        "app",
        "verify-all",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--format",
        "json",
    ]);
    assert!(healthy.status.success());
    let report: serde_json::Value = serde_json::from_slice(&healthy.stdout).unwrap();
    assert_eq!(report["verified"], 3);
    assert_eq!(report["mismatched"], 0);
    assert_eq!(report["errors"], 0);
    metadata::verify(fixture);
    missing::verify(fixture, first);
    tamper::tampering(fixture, first);
    cases::legacy(fixture, first);
}
