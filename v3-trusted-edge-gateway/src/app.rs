use crate::config::AppConfig;

pub fn capability_matches(app: &AppConfig, presented: &str) -> bool {
    let p = presented.as_bytes();
    // constant-time over same-length candidates; length mismatch is still rejected.
    let mut ok = false;
    for c in &app.edge_capabilities {
        let b = c.as_bytes();
        if crate::token::ct_eq(p, b) {
            ok = true;
        }
    }
    ok && !presented.is_empty()
}
