use super::{Event, EventScan, EventSet, EventStore, page};
use anyhow::{Result, bail};

impl<S: EventStore> EventSet<'_, S> {
    /// Bounded live enumeration, NOT an atomic snapshot.
    pub fn scan(&self, max_pages: usize) -> Result<EventScan> {
        if !(1..=100).contains(&max_pages) {
            bail!("max_pages must be 1..=100");
        }
        let matcher = format!("{}.*", self.prefix);
        let mut after: Option<String> = None;
        let mut events = Vec::new();
        for _ in 0..max_pages {
            let page = self.store.event_page(&matcher, after.as_deref())?;
            page::validate(self, &page, after.as_deref())?;
            for key in &page.keys {
                let bytes = self
                    .store
                    .get(key)?
                    .ok_or_else(|| anyhow::anyhow!("listed event disappeared"))?;
                let id = key.strip_prefix(&(self.prefix.clone() + ".")).unwrap();
                events.push(Event {
                    id: id.into(),
                    bytes,
                });
            }
            match page.next {
                None => {
                    return Ok(EventScan {
                        events,
                        complete: true,
                    });
                }
                Some(next) => after = Some(next),
            }
        }
        Ok(EventScan {
            events,
            complete: false,
        })
    }
}
