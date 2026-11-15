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

#[cfg(unix)]
#[test]
fn credential_operations_serialize_on_a_stable_sidecar_lock() {
    use super::lock;
    use std::{sync::mpsc, thread, time::Duration};
    let dir = std::env::temp_dir().join(format!("skrynia-lock-{:032x}", rand::random::<u128>()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("credential.json");
    let first = lock(&path).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let second_path = path.clone();
    let worker = thread::spawn(move || {
        started_tx.send(()).unwrap();
        let _guard = lock(&second_path).unwrap();
        done_tx.send(()).unwrap();
    });
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(done_rx.recv_timeout(Duration::from_millis(75)).is_err());
    drop(first);
    done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.join().unwrap();
    fs::remove_dir_all(dir).unwrap();
}
