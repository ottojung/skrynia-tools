use super::{
    model::AppConfig,
    model::Config,
    normalize::{normalize_host, normalize_origin},
};

impl Config {
    pub fn find_app(&self, host: &str, origin: &str) -> Option<&AppConfig> {
        let host = normalize_host(host)?;
        let origin = normalize_origin(origin)?;
        self.apps.iter().find(|app| {
            app.hosts
                .iter()
                .any(|h| normalize_host(h).as_deref() == Some(host.as_str()))
                && app
                    .origins
                    .iter()
                    .any(|o| normalize_origin(o).as_deref() == Some(origin.as_str()))
        })
    }
}
