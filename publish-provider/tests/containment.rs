//! Containment and non-regular-input regressions.

mod fifo;
mod support;

use support::{Fixture, call, request};

#[test]
fn rejects_fifo_and_directory_mailbox_tickets() {
    let f = Fixture::start();
    fifo::mkfifo(&f.requests.join("0000000a"));
    let (tx, rx) = std::sync::mpsc::channel();
    let base = f.base.clone();
    std::thread::spawn(move || {
        tx.send(support::call_at(&base, "publish", 10)).ok();
    });
    let code = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("fifo must not block");
    assert_eq!(code, 1);
    std::fs::create_dir(f.requests.join("0000000b")).expect("dir ticket");
    assert_eq!(call(&f, "publish", 11), 1);
}

#[test]
fn rejects_fifo_source_without_blocking() {
    let f = Fixture::start();
    std::fs::create_dir_all(f.disk.join("apps")).expect("apps");
    fifo::mkfifo(&f.disk.join("apps/fifo.txt"));
    request(&f, 12, b"apps/fifo.txt");
    let (tx, rx) = std::sync::mpsc::channel();
    let base = f.base.clone();
    std::thread::spawn(move || {
        tx.send(support::call_at(&base, "publish", 12)).ok();
    });
    let code = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("source fifo must not block");
    assert_eq!(code, 3);
}
