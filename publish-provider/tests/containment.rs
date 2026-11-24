//! Containment and non-regular-input regressions.

mod support;

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::time::Duration;
use support::{Fixture, call, request};

fn mkfifo(path: &Path) {
    let name = CString::new(path.as_os_str().as_bytes()).expect("cstr");
    let rc = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(rc, 0, "mkfifo failed");
    assert!(
        std::fs::symlink_metadata(path)
            .expect("meta")
            .file_type()
            .is_fifo()
    );
}

#[test]
fn rejects_fifo_and_directory_mailbox_tickets() {
    let f = Fixture::start();
    let fifo = f.requests.join("0000000a");
    mkfifo(&fifo);
    let (tx, rx) = std::sync::mpsc::channel();
    let base = f.base.clone();
    std::thread::spawn(move || {
        tx.send(support::call_at(&base, "publish", 10)).ok();
    });
    let code = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("fifo must not block");
    assert_eq!(code, 1);

    std::fs::create_dir(f.requests.join("0000000b")).expect("dir ticket");
    assert_eq!(call(&f, "publish", 11), 1);
}

#[test]
fn rejects_fifo_source_without_blocking() {
    let f = Fixture::start();
    std::fs::create_dir_all(f.disk.join("apps")).expect("apps");
    mkfifo(&f.disk.join("apps/fifo.txt"));
    request(&f, 12, b"apps/fifo.txt");
    let (tx, rx) = std::sync::mpsc::channel();
    let base = f.base.clone();
    std::thread::spawn(move || {
        tx.send(support::call_at(&base, "publish", 12)).ok();
    });
    let code = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("source fifo must not block");
    assert_eq!(code, 3);
}

#[test]
fn roots_reject_both_containment_directions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requests = dir.path().join("requests");
    let disk = dir.path().join("disk");
    let nginx_inside_disk = disk.join("nginx");
    std::fs::create_dir(&requests).expect("requests");
    std::fs::create_dir(&disk).expect("disk");
    std::fs::create_dir(&nginx_inside_disk).expect("nginx inside disk");
    let err = skrynia_publish_provider::roots::open(&requests, &disk, &nginx_inside_disk)
        .err()
        .expect("nginx inside disk must be rejected");
    assert!(err.contains("must not overlap"), "{err}");

    let dir2 = tempfile::tempdir().expect("tempdir2");
    let nginx = dir2.path().join("nginx");
    let disk_inside_nginx = nginx.join("disk");
    let requests2 = dir2.path().join("requests");
    std::fs::create_dir(&nginx).expect("nginx");
    std::fs::create_dir(&disk_inside_nginx).expect("disk inside nginx");
    std::fs::create_dir(&requests2).expect("requests2");
    let err = skrynia_publish_provider::roots::open(&requests2, &disk_inside_nginx, &nginx)
        .err()
        .expect("disk inside nginx must be rejected");
    assert!(err.contains("must not overlap"), "{err}");
}
