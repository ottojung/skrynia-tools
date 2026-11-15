mod support;
use std::fs;
use support::Fixture;

#[test]
fn stale_work_recovered_without_touching_other_apps() {
    let fixture = Fixture::new();
    let work = fixture.apps.join(".work");
    fs::create_dir_all(&work).unwrap();
    let abandoned = work.join(format!("hello-{}", "0".repeat(32)));
    let another = work.join(format!("hello-other-{}", "1".repeat(32)));
    fs::create_dir(&abandoned).unwrap();
    fs::create_dir(&another).unwrap();
    let commit = fixture.commit("recovered");
    let result = fixture.deploy(&commit);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!abandoned.exists());
    assert!(another.is_dir());
}
#[cfg(unix)]
#[test]
fn refuses_symlinked_deployment_directory() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let victim = fixture.root.join("victim");
    fs::create_dir(&victim).unwrap();
    fs::create_dir(&fixture.apps).unwrap();
    symlink(&victim, fixture.apps.join("releases")).unwrap();
    let commit = fixture.commit("safe");
    let result = fixture.deploy(&commit);
    assert!(!result.status.success());
    assert_eq!(fs::read_dir(victim).unwrap().count(), 0);
}
