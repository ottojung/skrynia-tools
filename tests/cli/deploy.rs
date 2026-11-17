use super::helpers::cli;
#[test]
fn deploy_requires_one_source_and_git_commit() {
    let no_source = cli(&[
        "app",
        "deploy",
        "--name",
        "demo",
        "--url",
        "http://localhost:1",
    ]);
    assert_eq!(no_source.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&no_source.stderr).contains("--directory"));

    let missing_commit = cli(&[
        "app",
        "deploy",
        "--name",
        "demo",
        "--url",
        "http://localhost:1",
        "--repo",
        "https://example.org/repo",
    ]);
    assert_eq!(missing_commit.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing_commit.stderr).contains("--commit"));

    let both = cli(&[
        "app",
        "deploy",
        "--name",
        "demo",
        "--url",
        "http://localhost:1",
        "--repo",
        "https://example.org/repo",
        "--commit",
        "abc",
        "--directory",
        "/tmp/built",
    ]);
    assert_eq!(both.status.code(), Some(2));
}
