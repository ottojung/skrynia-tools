//! Concurrent tickets converge without duplicate or dangling publications.

mod support;

use support::{Fixture, call_at, request_at, source, target};

#[test]
fn concurrent_publish_and_unpublish_are_idempotent() {
    let f = Fixture::start();
    let rel = "apps/concurrent.txt";
    source(&f, rel, b"x");
    let base = f.base.clone();
    let mailbox = f.requests.clone();

    std::thread::scope(|scope| {
        for ticket in 100..108u32 {
            let base = base.clone();
            let mailbox = mailbox.clone();
            scope.spawn(move || {
                request_at(&mailbox, ticket, rel.as_bytes());
                assert_eq!(call_at(&base, "publish", ticket), 0);
            });
        }
    });
    assert!(target(&f, rel).is_some());

    std::thread::scope(|scope| {
        for ticket in 100..108u32 {
            let base = base.clone();
            scope.spawn(move || assert_eq!(call_at(&base, "unpublish", ticket), 0));
        }
    });
    assert!(target(&f, rel).is_none());
}
