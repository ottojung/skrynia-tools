use super::{CreateResult, EventPage, EventSet, EventStore, Store};
use anyhow::Result;

struct Broken;
impl Store for Broken {
    fn get(&self, _: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }
    fn create(&self, _: &str, _: &[u8]) -> Result<CreateResult> {
        anyhow::bail!("transport failed before write")
    }
}
impl EventStore for Broken {
    fn event_page(&self, _: &str, _: Option<&str>) -> Result<EventPage> {
        Ok(EventPage {
            keys: vec![],
            next: Some("events.abc".into()),
        })
    }
}
#[test]
fn rejected_empty_cursor_and_uncertain_missing_create() {
    let store = Broken;
    let set = EventSet::open(&store, "events").unwrap();
    assert!(
        set.insert("11111111111111111111111111111111", b"data")
            .unwrap_err()
            .to_string()
            .contains("uncertain")
    );
    assert!(set.scan(2).unwrap_err().to_string().contains("cursor"));
}
