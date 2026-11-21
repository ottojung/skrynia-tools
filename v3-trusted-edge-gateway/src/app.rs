pub fn capability_matches(app: &super::config::AppConfig, presented: &str) -> bool {
    let p = presented.as_bytes();
    let mut ok = false;
    for c in &app.edge_capabilities {
        let b = c.as_bytes();
        if super::token::ct_eq(p, b) {
            ok = true;
        }
    }
    ok && !presented.is_empty()
}
