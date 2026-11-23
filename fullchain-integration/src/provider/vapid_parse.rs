//! Splits a VAPID authorization header into its token and key parts.
pub fn parse(authz: &str) -> Result<(&str, &str), &'static str> {
    let a = authz
        .strip_prefix("vapid ")
        .or_else(|| authz.strip_prefix("Vapid "))
        .ok_or("scheme")?;
    let mut t = None;
    let mut k = None;
    for part in a.split(',') {
        let p = part.trim();
        if let Some(v) = p.strip_prefix("t=") {
            t = Some(v.trim());
        }
        if let Some(v) = p.strip_prefix("k=") {
            k = Some(v.trim());
        }
    }
    Ok((t.ok_or("t")?, k.ok_or("k")?))
}
