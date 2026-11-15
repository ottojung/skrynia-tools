use super::copy_tree;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn publication_preserves_executable_artifacts_without_privileged_bits() {
    let root = std::env::temp_dir().join(format!(
        "skrynia-executable-{:032x}",
        rand::random::<u128>()
    ));
    let source = root.join("source");
    let published = root.join("published");
    fs::create_dir_all(&source).unwrap();
    let program = source.join("program");
    fs::write(&program, b"program bytes").unwrap();
    #[cfg(unix)]
    fs::set_permissions(&program, fs::Permissions::from_mode(0o4755)).unwrap();
    copy_tree(&source, &published).unwrap();
    assert_eq!(
        fs::read(published.join("program")).unwrap(),
        b"program bytes"
    );
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(published.join("program"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o755
    );
    fs::remove_dir_all(root).unwrap();
}
