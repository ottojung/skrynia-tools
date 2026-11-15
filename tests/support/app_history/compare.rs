use super::Fixture;

pub fn verify(fixture: &Fixture, first: &serde_json::Value, one: &str, three: &str) {
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
    let compare = fixture.cli(&[
        "app",
        "compare",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--from",
        first["id"].as_str().unwrap(),
        "--to",
        all[0]["id"].as_str().unwrap(),
        "--format",
        "json",
    ]);
    let compared: serde_json::Value = serde_json::from_slice(&compare.stdout).unwrap();
    assert_eq!(compared["metadata_only"], true);
    assert_eq!(compared["from"]["commit"], one);
    assert_eq!(compared["to"]["commit"], three);
}
