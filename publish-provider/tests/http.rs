//! Real HTTP against the compiled service binary.

mod support;

use support::{Fixture, call, client, request, source, target};

#[test]
fn version_advertises_both_syscalls() {
    let f = Fixture::start();
    let value: serde_json::Value = client()
        .post(format!("{}/version", f.base))
        .send()
        .expect("version")
        .json()
        .expect("json");
    assert_eq!(
        value["syscalls"],
        serde_json::json!(["publish", "unpublish"])
    );
    assert_eq!(
        client()
            .get(format!("{}/version", f.base))
            .send()
            .expect("get")
            .status(),
        404
    );
}

#[test]
fn publish_and_unpublish_are_idempotent() {
    let f = Fixture::start();
    let rel = "apps/hello/index.html";
    let src = source(&f, rel, b"hello");
    request(&f, 42, rel.as_bytes());
    assert_eq!(call(&f, "publish", 42), 0);
    assert_eq!(target(&f, rel).as_deref(), Some(src.as_path()));
    assert_eq!(
        std::fs::read(f.nginx.join(rel)).expect("served bytes"),
        b"hello"
    );
    assert_eq!(call(&f, "publish", 42), 0);
    assert_eq!(call(&f, "unpublish", 42), 0);
    assert!(target(&f, rel).is_none());
    assert!(src.is_file());
    assert_eq!(call(&f, "unpublish", 42), 0);
}
