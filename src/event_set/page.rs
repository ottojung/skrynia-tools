use super::{EventPage, EventSet, EventStore};
use anyhow::{Result, bail};

pub fn validate<S: EventStore>(
    set: &EventSet<'_, S>,
    page: &EventPage,
    after: Option<&str>,
) -> Result<()> {
    if page.keys.len() > 1000 {
        bail!("event listing page exceeds Skrynia limit");
    }
    let mut previous = after;
    for key in &page.keys {
        let id = key
            .strip_prefix(&(set.prefix.clone() + "."))
            .ok_or_else(|| anyhow::anyhow!("event listing returned outside scope"))?;
        set.key(id)?;
        if previous.is_some_and(|last| key.as_str() <= last) {
            bail!("event listing keys not strictly increasing");
        }
        previous = Some(key);
    }
    if let Some(next) = page.next.as_deref()
        && (page.keys.is_empty() || Some(next) != previous)
    {
        bail!("event pagination cursor is invalid");
    }
    Ok(())
}
