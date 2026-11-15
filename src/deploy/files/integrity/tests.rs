use super::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[test]
fn tree_hash_detects_bytes_names_empty_directories_and_symlinks() {
    let root =
        std::env::temp_dir().join(format!("skrynia-integrity-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a"), b"hello").unwrap();
    let original = digest_tree(&root).unwrap();
    fs::write(root.join("a"), b"world").unwrap();
    assert_ne!(original, digest_tree(&root).unwrap());
    fs::write(root.join("a"), b"hello").unwrap();
    assert_eq!(original, digest_tree(&root).unwrap());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = root.join("a");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        assert_ne!(original, digest_tree(&root).unwrap());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    }
    fs::create_dir(root.join("empty")).unwrap();
    assert_ne!(original, digest_tree(&root).unwrap());
    fs::rename(root.join("a"), root.join("b")).unwrap();
    assert_ne!(original, digest_tree(&root).unwrap());
    #[cfg(unix)]
    {
        symlink("b", root.join("link")).unwrap();
        assert!(digest_tree(&root).is_err());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stable_release_tree_digest_fixture() {
    let root = std::env::temp_dir().join(format!(
        "skrynia-hash-fixture-{:032x}",
        rand::random::<u128>()
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("a"), b"hello").unwrap();
    assert_eq!(
        digest_tree(&root).unwrap(),
        "3024c1b06c7d8ca9229939c0d99fd3017b02c9c9c3e5019d20fd7c2dd0a74256"
    );
    fs::remove_dir_all(root).unwrap();
}
