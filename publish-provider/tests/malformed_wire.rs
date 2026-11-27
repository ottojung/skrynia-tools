//! Wrong signatures must be rejected before publication side effects.
mod support;
use support::{Fixture, call, client, request, source, target};

#[test]
fn malformed_typed_requests_never_publish_or_unpublish() {
    let f = Fixture::start();
    let rel = "apps/hello/index.html";
    source(&f, rel, b"hello");
    request(&f, 42, rel.as_bytes());
    let wrong = [
        &[][..],
        &[42, 0, 0, 0],
        &[2, 1, 0x32, 42, 0, 0, 0],
        &[1, 2, 0x32, 42, 0, 0, 0],
        &[1, 1, 0x40, 42, 0, 0, 0, 0, 0, 0, 0],
        &[1, 1, 0x32, 42, 0, 0],
        &[1, 1, 0x32, 42, 0, 0, 0, 0],
    ];
    for body in wrong {
        let response = client()
            .post(format!("{}/publish", f.base))
            .body(body.to_vec())
            .send()
            .unwrap();
        assert_eq!(response.status(), 400);
        assert!(target(&f, rel).is_none());
    }
    assert_eq!(call(&f, "publish", 42), 0);
    for body in wrong {
        let response = client()
            .post(format!("{}/unpublish", f.base))
            .body(body.to_vec())
            .send()
            .unwrap();
        assert_eq!(response.status(), 400);
        assert!(target(&f, rel).is_some());
    }
    assert_eq!(call(&f, "unpublish", 42), 0);
}
