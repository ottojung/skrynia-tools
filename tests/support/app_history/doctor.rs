use super::Fixture;

pub fn verify(fixture: &Fixture) {
    let result = fixture.cli(&[
        "app",
        "doctor",
        "--apps-dir",
        fixture.apps.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["prerequisites_ok"], true);
    assert_eq!(report["git_available"], true);
    assert_eq!(report["podman_rootless"], true);
    assert!(report["missing_directories"].as_array().unwrap().is_empty());
}
