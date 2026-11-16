use super::Fixture;
use std::fs;
pub fn verify(fixture: &Fixture) {
    let root = fixture.apps.to_str().unwrap();
    let metadata = fixture.apps.join("metadata/hello");
    for filename in ["bad-name.txt", "ffffffffffff-0000000000000000.json"] {
        let path = metadata.join(filename);
        fs::write(&path, b"{invalid").unwrap();
        let output = fixture.cli(&[
            "app",
            "verify-all",
            "--name",
            "hello",
            "--apps-dir",
            root,
            "--format",
            "json",
        ]);
        assert!(!output.status.success());
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["verified"], 3);
        assert_eq!(report["errors"], 1);
        assert_eq!(report["releases"].as_array().unwrap().len(), 4);
        assert!(
            report["releases"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["error"].is_string())
        );
        fs::remove_file(path).unwrap();
    }
}
