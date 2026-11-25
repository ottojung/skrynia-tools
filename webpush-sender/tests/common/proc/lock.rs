//! Hostile lock shapes are refused without mutating the target inode.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use webpush_sender::push::proc::Lock;

pub fn hostile() {
    let dir = std::env::temp_dir().join(format!("rt-lockh-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let lock = dir.join("sender.lock");
    let target = dir.join("target");
    fs::write(&target, b"preserve").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap();
    std::os::unix::fs::symlink(&target, &lock).unwrap();
    assert!(Lock::acquire(&dir).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"preserve");
    fs::remove_file(&lock).unwrap();
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&lock)
            .status()
            .unwrap()
            .success()
    );
    assert!(Lock::acquire(&dir).is_err());
    fs::remove_file(&lock).unwrap();
    fs::hard_link(&target, &lock).unwrap();
    assert!(Lock::acquire(&dir).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"preserve");
    fs::remove_file(&lock).unwrap();
    fs::write(&lock, b"").unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(Lock::acquire(&dir).is_err());
    assert_eq!(
        fs::metadata(&lock).unwrap().permissions().mode() & 0o777,
        0o644
    );
    fs::remove_file(&lock).unwrap();
    let held = Lock::acquire(&dir).unwrap();
    assert!(Lock::acquire(&dir).is_err());
    drop(held);
    assert!(Lock::acquire(&dir).is_ok());
    let _ = fs::remove_dir_all(&dir);
}
