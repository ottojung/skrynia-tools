use super::Fixture;
use std::fs;

pub fn verify(fixture: &Fixture, first: &serde_json::Value) {
    let dir = fixture.apps.join("metadata/hello");
    let source = dir.join(format!("{}.json", first["id"].as_str().unwrap()));
    let mut record: serde_json::Value = serde_json::from_slice(&fs::read(source).unwrap()).unwrap();
    let id = "ffffffffffff-0000000000000001";
    record["id"] = id.into();
    record["commit"] = format!("ffffffffffff{}", "a".repeat(28)).into();
    let path = dir.join(format!("{id}.json"));
    fs::write(&path, serde_json::to_vec(&record).unwrap()).unwrap();
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
    assert!(!output.status.success());
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["verified"], 3);
    assert_eq!(data["errors"], 1);
    assert!(
        data["releases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["release"] == id && item["error"].is_string())
    );
    fs::remove_file(path).unwrap();
}
