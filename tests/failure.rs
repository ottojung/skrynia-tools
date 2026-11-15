mod support;
use support::Fixture;

#[test]
fn failed_build_never_changes_active_release() {
    let fixture = Fixture::new();
    let good = fixture.commit("valid");
    assert!(fixture.deploy(&good).status.success());
    fixture.break_build();
    let broken = fixture.commit("invalid");
    let result = fixture.deploy(&broken);
    assert!(!result.status.success());
    let active = fixture.apps.join("apps").join("hello").join("index.html");
    assert_eq!(std::fs::read_to_string(active).unwrap().trim(), "valid");
}
