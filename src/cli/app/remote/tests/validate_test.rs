use super::validate;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn rejects_unsafe_ssh_identifiers_and_paths() {
    for host in ["", "-option", "host.", "bad host", "host\noption"] {
        assert!(validate::host(host).is_err());
    }
    for user in ["", "-operator", "bad:user", "bad user"] {
        assert!(validate::user(user).is_err());
    }
    validate::host("127.0.0.1").unwrap();
    validate::user("_deployer").unwrap();
    let root = std::env::temp_dir().join(format!("skrynia-ssh-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    let key = root.join("key");
    fs::write(&key, b"private test data").unwrap();
    #[cfg(unix)]
    {
        fs::set_permissions(&key, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(validate::file(key.to_str().unwrap(), true).is_err());
        fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&key, root.join("link")).unwrap();
        assert!(validate::file(root.join("link").to_str().unwrap(), true).is_err());
    }
    validate::file(key.to_str().unwrap(), true).unwrap();
    fs::remove_dir_all(root).unwrap();
}
