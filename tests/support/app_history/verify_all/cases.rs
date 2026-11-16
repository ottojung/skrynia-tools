use super::Fixture;
use std::fs;

pub(super) fn invoke(fixture: &Fixture) -> (bool, serde_json::Value) {
    let output = fixture.cli(&[
        "app",
        "verify-all",
        "--name",
        "hello",
        "--apps-dir",
        fixture.apps.to_str().unwrap(),
        "--format",
        "json",
    ]);
    (
        output.status.success(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
pub fn legacy(fixture: &Fixture, first: &serde_json::Value) {
    let path = fixture
        .apps
        .join("metadata/hello")
        .join(format!("{}.json", first["id"].as_str().unwrap()));
    let original = fs::read(&path).unwrap();
    let mut metadata: serde_json::Value = serde_json::from_slice(&original).unwrap();
    metadata.as_object_mut().unwrap().remove("content_sha256");
    fs::write(&path, serde_json::to_vec(&metadata).unwrap()).unwrap();
    let (status, report) = invoke(fixture);
    assert!(!status);
    assert_eq!(report["unverifiable"], 1);
    assert_eq!(report["verified"], 2);
    fs::write(path, original).unwrap();
    assert!(invoke(fixture).0);
}
