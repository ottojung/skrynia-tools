use super::{Authority, CompatError, Usage, scope};
use std::collections::HashMap;
impl Authority {
    pub fn check_write_quota(
        &self,
        acting: &str,
        key: &str,
        objects: &HashMap<String, u64>,
        new_size: u64,
        subscriptions: &[(String, u64)],
    ) -> Result<(), CompatError> {
        let err = || CompatError::new(507, "grant_quota_exceeded");
        let mut ancestor = Some(acting);
        let mut steps = 0;
        while let Some(id) = ancestor {
            let grant = self.grants.get(id).ok_or_else(err)?;
            let mut usage = self.usage(id, objects, subscriptions).ok_or_else(err)?;
            if scope::matches(&grant.locations, key) {
                if let Some(old) = objects.get(key) {
                    usage = usage
                        .checked_sub(Usage::object(*old).ok_or_else(err)?)
                        .ok_or_else(err)?;
                }
                usage = usage
                    .checked_add(Usage::object(new_size).ok_or_else(err)?)
                    .ok_or_else(err)?;
            }
            if grant.max_bytes.is_some_and(|v| usage.bytes > v)
                || grant.max_objects.is_some_and(|v| usage.objects > v)
            {
                return Err(err());
            }
            ancestor = grant.parent.as_deref();
            steps += 1;
            if steps > self.grants.len() {
                return Err(err());
            }
        }
        Ok(())
    }
}
