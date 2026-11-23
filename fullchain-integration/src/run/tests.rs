use super::*;
#[test]
fn existing_directory_keeps_its_contents() {
    let base = create(None).unwrap();
    let occupied = base.join("keep");
    fs::create_dir(&occupied).unwrap();
    let sentinel = occupied.join("important");
    fs::write(&sentinel, b"keep these bytes").unwrap();
    let e = create(Some(occupied.clone())).unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&sentinel).unwrap(), b"keep these bytes");
    fs::remove_file(sentinel).unwrap();
    fs::remove_dir(occupied).unwrap();
    fs::remove_dir(base).unwrap();
}
#[test]
fn defaults_are_fresh_and_private() {
    let a = create(None).unwrap();
    let b = create(None).unwrap();
    assert_ne!(a, b);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&a).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&b).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    fs::remove_dir(a).unwrap();
    fs::remove_dir(b).unwrap();
}
