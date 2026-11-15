mod support;
use support::Fixture;

#[test]
fn immutable_versions_activate_and_rollback() {
    let fixture = Fixture::new();
    let first = fixture.commit("first version");
    let a = fixture.deploy(&first);
    assert!(
        a.status.success(),
        "{:?}",
        String::from_utf8_lossy(&a.stderr)
    );
    let previous: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    let active = fixture.apps.join("apps").join("hello").join("index.html");
    assert_eq!(
        std::fs::read_to_string(&active).unwrap().trim(),
        "first version"
    );
    let second = fixture.commit("second version");
    assert!(fixture.deploy(&second).status.success());
    assert_eq!(
        std::fs::read_to_string(&active).unwrap().trim(),
        "second version"
    );
    let result = fixture.cli(&[
        "app",
        "rollback",
        "--name",
        "hello",
        "--release",
        previous["id"].as_str().unwrap(),
        "--apps-dir",
        fixture.apps.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&active).unwrap().trim(),
        "first version"
    );
}

#[test]
fn developer_version_history_cli() {
    support::app_history::verify();
}
