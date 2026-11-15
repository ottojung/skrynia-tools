use super::Fixture;

pub fn verify(fixture: &Fixture, first: &serde_json::Value) {
    let root = fixture.apps.to_str().unwrap();
    let id = first["id"].as_str().unwrap();
    let rollback = fixture.cli(&[
        "app",
        "rollback",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--release",
        id,
        "--format",
        "json",
    ]);
    assert!(rollback.status.success());
    let response: serde_json::Value = serde_json::from_slice(&rollback.stdout).unwrap();
    assert_eq!(response["activated"], id);
    assert_eq!(
        std::fs::read_to_string(fixture.apps.join("apps/hello/index.html"))
            .unwrap()
            .trim(),
        "version one"
    );
    let current = fixture.cli(&["app", "current", "--name", "hello", "--apps-dir", root]);
    assert!(String::from_utf8(current.stdout).unwrap().contains(id));
    let invalid = fixture.cli(&[
        "app",
        "show",
        "--name",
        "hello",
        "--apps-dir",
        root,
        "--release",
        "000000000000-0000000000000000",
    ]);
    assert!(!invalid.status.success());
}
