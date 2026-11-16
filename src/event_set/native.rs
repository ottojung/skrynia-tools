use super::{EventPage, EventStore, SkryniaClient};
use anyhow::Result;
impl EventStore for SkryniaClient {
    fn event_page(&self, matcher: &str, after: Option<&str>) -> Result<EventPage> {
        Ok(serde_json::from_value(self.list_page(matcher, after)?)?)
    }
}
