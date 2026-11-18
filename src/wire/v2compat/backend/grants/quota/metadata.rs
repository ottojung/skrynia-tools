use super::{Authority, Usage};
pub fn for_grant(auth: &Authority, id: &str, subscriptions: &[(String, u64)]) -> Option<Usage> {
    let mut used = Usage::default();
    for grant in auth.grants.values() {
        let mut cursor = Some(grant.id.as_str());
        let mut steps = 0;
        while let Some(current) = cursor {
            if current == id {
                let extra = (grant.users.len().saturating_sub(1) as u64)
                    .checked_mul(128)?
                    .checked_add(2048)?;
                used = used.checked_add(Usage {
                    bytes: extra,
                    objects: 0,
                })?;
                break;
            }
            cursor = auth.grants.get(current)?.parent.as_deref();
            steps += 1;
            if steps > auth.grants.len() {
                return None;
            }
        }
    }
    for (grant_id, count) in subscriptions {
        let mut cursor = Some(grant_id.as_str());
        let mut steps = 0;
        while let Some(current) = cursor {
            if current == id {
                used = used.checked_add(Usage {
                    bytes: count.checked_mul(4096)?,
                    objects: *count,
                })?;
                break;
            }
            cursor = auth.grants.get(current)?.parent.as_deref();
            steps += 1;
            if steps > auth.grants.len() {
                return None;
            }
        }
    }
    Some(used)
}
