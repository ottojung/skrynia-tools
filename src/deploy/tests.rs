use super::{
    files,
    types::DeploySpec,
    validate::{valid_name, valid_rel},
};
use std::path::PathBuf;

#[test]
fn source_paths_do_not_escape_checkout() {
    assert!(valid_rel("."));
    assert!(valid_rel("apps/frontend"));
    assert!(!valid_rel("../secrets"));
    assert!(!valid_rel("/etc/passwd"));
    assert!(!valid_name("../admin"));
}
#[test]
fn only_supported_github_repos() {
    let spec = DeploySpec {
        name: "hello".into(),
        repo: "https://github.com/ottojung/demo".into(),
        subdir: "apps/hello".into(),
        commit: None,
        apps_dir: PathBuf::from("/tmp"),
        builder_image: "builder".into(),
    };
    assert!(spec.validate().is_ok());
    let mut bad = spec.clone();
    bad.repo = "https://github.com/owner/repo?token=secret".into();
    assert!(bad.validate().is_err());
}
#[cfg(unix)]
#[test]
fn rejects_symlinked_build_output() {
    use std::os::unix::fs::symlink;
    let root = std::env::temp_dir().join(format!("skrynia-deploy-{:032x}", rand::random::<u128>()));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("src");
    let dest = root.join("dst");
    std::fs::create_dir(&source).unwrap();
    symlink("/etc/passwd", source.join("bad")).unwrap();
    assert!(files::copy_tree(&source, &dest).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
