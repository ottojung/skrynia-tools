use super::AppConfig;
use super::checks;
use std::collections::BTreeMap;

pub fn validate_apps(apps: &[AppConfig]) -> Result<(), String> {
    let mut seen_origin = BTreeMap::new();
    let mut seen_host = BTreeMap::new();
    let mut seen_cap = BTreeMap::new();
    for app in apps {
        if app.name.trim().is_empty() {
            return Err("app name is required".into());
        }
        if app.hosts.is_empty() || app.origins.is_empty() || app.edge_capabilities.is_empty() {
            return Err(format!(
                "app {} requires hosts, origins, and caps",
                app.name
            ));
        }
        checks::origins(app, &mut seen_origin)?;
        checks::hosts(app, &mut seen_host)?;
        checks::caps(app, &mut seen_cap)?;
    }
    Ok(())
}
