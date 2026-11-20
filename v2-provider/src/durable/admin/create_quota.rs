use skrynia_tools::wire::v2compat::{CompatError, grants::Authority};
fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn check_parent_meta(
    auth: &Authority,
    parent: &str,
    sizes: &std::collections::HashMap<String, u64>,
) -> Result<(), CompatError> {
    let mut cur = Some(parent);
    let mut steps = 0;
    while let Some(id) = cur {
        let g = auth
            .grant(id)
            .ok_or_else(|| err(507, "grant_quota_exceeded"))?;
        if let Some(limit) = g.max_bytes {
            let usage = auth
                .usage(id, sizes, &[])
                .ok_or_else(|| err(507, "grant_quota_exceeded"))?;
            if usage.bytes.saturating_add(2048) > limit {
                return Err(err(507, "grant_quota_exceeded"));
            }
        }
        cur = g.parent.as_deref();
        steps += 1;
        if steps > 1000 {
            return Err(err(507, "grant_quota_exceeded"));
        }
    }
    Ok(())
}
