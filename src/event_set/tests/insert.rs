use super::{EventSet, Insert, Store, fake::Fake};

#[test]
fn idempotent_retry_collision_and_uncertain_create() {
    let store = Fake::default();
    let events = EventSet::open(&store, "app.events").unwrap();
    let id = "0123456789abcdef0123456789abcdef";
    assert_eq!(events.insert(id, b"original").unwrap(), Insert::Created);
    assert_eq!(
        events.insert(id, b"original").unwrap(),
        Insert::AlreadyPresent
    );
    assert!(events.insert(id, b"changed").is_err());
    assert!(events.insert("NOTHEX", b"invalid").is_err());
    *store.fail_next.lock().unwrap() = true;
    assert_eq!(
        events
            .insert("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", b"x")
            .unwrap(),
        Insert::AlreadyPresent
    );
    assert_eq!(
        store
            .get("app.events.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Some(b"x".to_vec())
    );
}
