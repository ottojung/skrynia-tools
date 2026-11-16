use super::files;
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
