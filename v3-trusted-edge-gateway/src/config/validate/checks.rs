use super::{AppConfig, normalize_host, normalize_origin};
use std::collections::BTreeMap;

pub fn origins(app: &AppConfig, seen: &mut BTreeMap<String, String>) -> Result<(), String> {
    for o in &app.origins {
        let origin = normalize_origin(o).ok_or_else(|| format!("invalid origin {}", app.name))?;
        if let Some(prev) = seen.insert(origin.clone(), app.name.clone()) {
            if prev != app.name {
                return Err(format!("origin {origin} mapped to multiple apps"));
            }
        }
    }
    Ok(())
}

pub fn hosts(app: &AppConfig, seen: &mut BTreeMap<String, String>) -> Result<(), String> {
    for h in &app.hosts {
        let host = normalize_host(h).ok_or_else(|| format!("invalid host {}", app.name))?;
        if let Some(prev) = seen.insert(host.clone(), app.name.clone()) {
            if prev != app.name {
                return Err(format!("host {host} mapped to multiple apps"));
            }
        }
    }
    Ok(())
}

pub fn caps(app: &AppConfig, seen: &mut BTreeMap<String, String>) -> Result<(), String> {
    for c in &app.edge_capabilities {
        if c.is_empty() {
            return Err(format!("empty edge capability for app {}", app.name));
        }
        if let Some(prev) = seen.insert(c.clone(), app.name.clone()) {
            if prev != app.name {
                return Err("edge capability must be unique across apps".into());
            }
        }
    }
    Ok(())
}
