use super::*;
use std::{sync::mpsc, time::Duration};

#[test]
fn rollback_ignores_arbitrary_other_version_files() {
    let fixture = Fixture::new();
    let target = fixture.add(1);
    fixture.active(&target.id);
    for n in 2..=500 {
        let id = format!("{n:012x}-{n:016x}");
        fs::write(
            fixture
                .root
                .join("metadata/hello")
                .join(format!("{id}.json")),
            b"not-json",
        )
        .unwrap();
    }
    rollback(&fixture.root, "hello", &target.id).unwrap();
    assert_eq!(
        inspect(&fixture.root, "hello").unwrap().unwrap().id,
        target.id
    );
}
#[test]
fn rollback_never_waits_on_app_build_lock() {
    let fixture = Fixture::new();
    let target = fixture.add(1);
    let _build = process::build_lock(&fixture.root, "hello").unwrap();
    let path = fixture.root.clone();
    let id = target.id.clone();
    let (tx, rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        tx.send(rollback(&path, "hello", &id)).unwrap();
    });
    let outcome = rx.recv_timeout(Duration::from_secs(3));
    assert!(outcome.is_ok(), "rollback blocked behind build phase");
    outcome.unwrap().unwrap();
    worker.join().unwrap();
}
