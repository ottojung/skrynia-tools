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
    assert!(versions.status.success());
    let all: serde_json::Value = serde_json::from_slice(&versions.stdout).unwrap();
    assert_eq!(all.as_array().unwrap().len(), 3);
    assert_eq!(all[0]["commit"], three);
    let history = fixture.cli(&["app", "history", "--name", "hello", "--apps-dir", root]);
    let printed = String::from_utf8(history.stdout).unwrap();
    assert!(printed.contains(first["id"].as_str().unwrap()));
    assert!(printed.contains("DEPLOYED"));
    let inventory = fixture.cli(&["app", "list", "--apps-dir", root, "--format", "json"]);
    let apps: serde_json::Value = serde_json::from_slice(&inventory.stdout).unwrap();
    assert_eq!(apps[0]["name"], "hello");
    assert_eq!(apps[0]["active"]["commit"], three);
    let show = fixture.cli(&[
        "app",
        "show",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--release",
        first["id"].as_str().unwrap(),
    ]);
    let shown: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(shown["commit"], one);
}
