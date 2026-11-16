use super::*;
#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(unix)]
#[test]
fn release_audit_reports_interrupted_activation_and_build_without_removing_them() {
    let fixture = Fixture::new();
    let active = fixture.add(1);
    fixture.active(&active.id);
    let candidate = fixture
        .root
        .join("apps/.hello.0123456789abcdef0123456789abcdef");
    symlink(format!("../releases/hello/{}", active.id), &candidate).unwrap();
    let workspace = fixture
        .root
        .join(".work/hello-0123456789abcdef0123456789abcdef");
    fs::create_dir(&workspace).unwrap();
    let report = super::super::doctor::audit(&fixture.root, "hello").unwrap();
    assert!(!report.clean);
    assert_eq!(report.abandoned_activation_links.len(), 1);
    assert_eq!(report.abandoned_workspaces.len(), 1);
    assert!(candidate.is_symlink());
    assert!(workspace.is_dir());
    fs::remove_file(candidate).unwrap();
    fs::remove_dir(workspace).unwrap();
    assert!(
        super::super::doctor::audit(&fixture.root, "hello")
            .unwrap()
            .clean
    );
}
