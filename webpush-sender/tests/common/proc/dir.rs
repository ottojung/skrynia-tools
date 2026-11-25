//! Ledger directory validation rejects symlink, foreign mode and non-dir.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use webpush_sender::push::proc::validate_ledger_dir;

pub fn dir_checks() {
    let root = std::env::temp_dir().join(format!("rt-dir-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let d = root.join("private");
    fs::create_dir_all(&d).unwrap();
    fs::set_permissions(&d, fs::Permissions::from_mode(0o700)).unwrap();
    validate_ledger_dir(&d).unwrap();
    fs::set_permissions(&d, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(validate_ledger_dir(&d).is_err());
    fs::set_permissions(&d, fs::Permissions::from_mode(0o700)).unwrap();
    let link = root.join("link");
    std::os::unix::fs::symlink(&d, &link).unwrap();
    assert!(validate_ledger_dir(&link).is_err());
    let _ = fs::remove_dir_all(&root);
}
