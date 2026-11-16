use super::{Store, wire};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

pub trait EventStore: Store {
    fn event_page(&self, matcher: &str, after: Option<&str>) -> Result<EventPage>;
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventPage {
    pub keys: Vec<String>,
    #[serde(default)]
    pub next: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub id: String,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventScan {
    pub events: Vec<Event>,
    pub complete: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Insert {
    Created,
    AlreadyPresent,
}

pub struct EventSet<'a, S: EventStore> {
    pub(super) store: &'a S,
    pub(super) prefix: String,
}
impl<'a, S: EventStore> EventSet<'a, S> {
    pub fn open(store: &'a S, prefix: &str) -> Result<Self> {
        if !wire::valid_prefix(prefix) || prefix.len() > 223 {
            bail!("invalid event-set prefix");
        }
        Ok(Self {
            store,
            prefix: prefix.to_owned(),
        })
    }
}
