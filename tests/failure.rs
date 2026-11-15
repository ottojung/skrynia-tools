mod support;
use support::Fixture;

#[test]
fn failed_build_never_changes_active_release() {
    let fixture = Fixture::new();
    let good = fixture.commit("valid");
    assert!(fixture.deploy(&good).status.success());
    std::fs::write(
        fixture.source.join("apps/site/Makefile"),
        "build:\n\tfalse\n",
    )
    .unwrap();
    let broken = fixture.commit("invalid");
    assert!(!fixture.deploy(&broken).status.success());
    let active = fixture.apps.join("apps").join("hello").join("index.html");
    assert_eq!(std::fs::read_to_string(active).unwrap().trim(), "valid");
}

#[test]
fn artifact_only_versions_can_be_rolled_back() {
    let fixture = Fixture::new();
    let original = fixture.commit("html site");
    assert!(fixture.deploy(&original).status.success());
    std::fs::write(
        fixture.source.join("apps/site/Makefile"),
        "build:\n\tmkdir -p build\n\tcp ../../shared.txt build/code.bin\n",
    )
    .unwrap();
    let binary_commit = fixture.commit("binary code");
    let binary = fixture.deploy(&binary_commit);
    assert!(binary.status.success());
    let release: serde_json::Value = serde_json::from_slice(&binary.stdout).unwrap();
    assert!(fixture.deploy(&original).status.success());
    let rollback = fixture.cli(&[
        "app",
        "rollback",
        "--name",
        "hello",
        "--release",
        release["id"].as_str().unwrap(),
        "--apps-dir",
        fixture.apps.to_str().unwrap(),
    ]);
    assert!(rollback.status.success());
    assert_eq!(
        std::fs::read_to_string(fixture.apps.join("apps/hello/code.bin"))
            .unwrap()
            .trim(),
        "binary code"
    );
}
