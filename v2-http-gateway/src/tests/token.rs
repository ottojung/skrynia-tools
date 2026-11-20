use super::super::token;
use std::path::PathBuf;
fn file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "skrynia-gateway-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ))
}
#[test]
fn token_file_must_be_exactly_32_bytes() {
    let path = file("len");
    for n in [0, 31, 33, 1024] {
        std::fs::write(&path, vec![5u8; n]).unwrap();
        assert!(token::load(&path).is_err());
    }
    std::fs::write(&path, [5u8; 32]).unwrap();
    #[cfg(unix)]
    super::private(&path);
    assert_eq!(token::load(&path).unwrap(), [5u8; 32]);
    std::fs::remove_file(path).unwrap();
}
#[cfg(unix)]
#[test]
fn token_file_rejects_broad_permissions_and_symlinks() {
    use std::os::unix::fs::symlink;
    let path = file("permissions");
    let link = file("link");
    std::fs::write(&path, [5u8; 32]).unwrap();
    assert!(token::load(&path).is_err());
    super::private(&path);
    symlink(&path, &link).unwrap();
    assert!(token::load(&link).is_err());
    std::fs::remove_file(link).unwrap();
    std::fs::remove_file(path).unwrap();
}
