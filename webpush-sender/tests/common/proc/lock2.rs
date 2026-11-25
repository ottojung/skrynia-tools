//! Real two-process lock contention on a persistent inode, isolated temp dir.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use webpush_sender::push::proc::Lock;

const ENV: &str = "SKRYNIA_LOCK_CHILD";
const DIR: &str = "SKRYNIA_LOCK_DIR";
const READY: &str = "SKRYNIA_LOCK_READY";

pub fn child() {
    if std::env::var(ENV).is_err() {
        return;
    }
    let binding = std::env::var(DIR).unwrap();
    let dir = Path::new(&binding);
    let _guard = Lock::acquire(dir).unwrap();
    fs::write(std::env::var(READY).unwrap(), b"1").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1200));
}

pub fn two_process() {
    let root = std::env::temp_dir().join(format!("rt-two-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let ready = root.join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("runtime_two_process_child")
        .arg("--exact")
        .env(ENV, "1")
        .env(DIR, &root)
        .env(READY, &ready)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    for _ in 0..200 {
        if ready.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(ready.exists(), "child did not acquire lock");
    assert!(Lock::acquire(&root).is_err());
    child.wait().unwrap();
    assert!(Lock::acquire(&root).is_ok());
    let _ = fs::remove_dir_all(&root);
}
