use super::Fixture;
pub fn verify(fixture: &Fixture, first: &serde_json::Value) {
    let root = fixture.apps.to_str().unwrap();
    let versions = fixture.cli(&[
        "app",
        "versions",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--format",
        "json",
    ]);
    let all: serde_json::Value = serde_json::from_slice(&versions.stdout).unwrap();
    let from = first["id"].as_str().unwrap();
    let to = all[0]["id"].as_str().unwrap();
    let output = fixture.cli(&[
        "app",
        "diff",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--from",
        from,
        "--to",
        to,
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    checks::validate(&report, from, to);
}

mod checks;
