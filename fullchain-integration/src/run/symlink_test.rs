use super::*;
#[cfg(unix)]
#[test]
fn existing_symlink_is_refused_without_following() {
    let base = create(None).unwrap();
    let protected = base.join("protected");
    fs::create_dir(&protected).unwrap();
    let sentinel = protected.join("sentinel");
    fs::write(&sentinel, b"do not change").unwrap();
    let link = base.join("redirect");
    std::os::unix::fs::symlink(&protected, &link).unwrap();
    assert_eq!(
        create(Some(link.clone())).unwrap_err().kind(),
        io::ErrorKind::AlreadyExists
    );
    assert_eq!(fs::read(&sentinel).unwrap(), b"do not change");
    fs::remove_file(link).unwrap();
    fs::remove_file(sentinel).unwrap();
    fs::remove_dir(protected).unwrap();
    fs::remove_dir(base).unwrap();
}
