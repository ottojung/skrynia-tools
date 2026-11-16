use super::{CreateResult, EventPage, EventStore, Store};
use anyhow::Result;
use std::{collections::BTreeMap, sync::Mutex};
#[derive(Default)]
pub struct Fake {
    pub items: Mutex<BTreeMap<String, Vec<u8>>>,
    pub fail_next: Mutex<bool>,
}
impl Store for Fake {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.items.lock().unwrap().get(key).cloned())
    }
    fn create(&self, key: &str, bytes: &[u8]) -> Result<CreateResult> {
        let mut items = self.items.lock().unwrap();
        if items.contains_key(key) {
            return Ok(CreateResult::Exists);
        }
        items.insert(key.into(), bytes.into());
        if std::mem::take(&mut *self.fail_next.lock().unwrap()) {
            anyhow::bail!("lost acknowledgment")
        }
        Ok(CreateResult::Created)
    }
}
impl EventStore for Fake {
    fn event_page(&self, matcher: &str, after: Option<&str>) -> Result<EventPage> {
        let prefix = matcher.strip_suffix('*').unwrap();
        let items = self.items.lock().unwrap();
        let keys = items
            .keys()
            .filter(|k| k.starts_with(prefix) && after.is_none_or(|v| k.as_str() > v))
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        let more = items
            .keys()
            .any(|k| k.starts_with(prefix) && keys.last().is_some_and(|last| k > last));
        let next = if more { keys.last().cloned() } else { None };
        Ok(EventPage { keys, next })
    }
}
