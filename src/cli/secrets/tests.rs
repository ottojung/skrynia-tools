use super::{read, replace, write_new};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn secrets_are_private_and_replaceable() {
    let dir = std::env::temp_dir().join(format!("skrynia-secret-{:032x}", rand::random::<u128>()));
    fs::create_dir(&dir).unwrap();
    let file = dir.join("token");
    write_new(&file, b"initial").unwrap();
    assert_eq!(read(file.to_str().unwrap()).unwrap(), "initial");
    replace(&file, b"updated").unwrap();
    assert_eq!(read(file.to_str().unwrap()).unwrap(), "updated");
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600
    );
    #[cfg(unix)]
    {
        symlink(&file, dir.join("link")).unwrap();
        assert!(read(dir.join("link").to_str().unwrap()).is_err());
    }
    fs::remove_dir_all(dir).unwrap();
}
