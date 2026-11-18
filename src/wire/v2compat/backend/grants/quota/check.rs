use super::{Authority, Usage, scope};
use std::collections::HashMap;
impl Authority {
    pub fn usage(
        &self,
        id: &str,
        objects: &HashMap<String, u64>,
        subscriptions: &[(String, u64)],
    ) -> Option<Usage> {
        let grant = self.grants.get(id)?;
        let mut used = super::metadata::for_grant(self, id, subscriptions)?;
        for (key, size) in objects {
            if scope::matches(&grant.locations, key) {
                used = used.checked_add(Usage::object(*size)?)?;
            }
        }
        Some(used)
    }
}
