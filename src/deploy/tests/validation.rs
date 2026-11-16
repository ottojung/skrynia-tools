use super::{DeploySpec, valid_name, valid_rel};
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
    bad.repo = spec.repo;
    for image in [
        "--privileged",
        "--network=host",
        "",
        "image with whitespace",
        "/absolute/path",
        "registry@bad!digest",
    ] {
        bad.builder_image = image.into();
        assert!(
            bad.validate().is_err(),
            "unsafe builder image accepted: {image}"
        );
    }
    bad.builder_image = "docker.io/library/rust@sha256:0123456789abcdef".into();
    assert!(bad.validate().is_ok());
}
