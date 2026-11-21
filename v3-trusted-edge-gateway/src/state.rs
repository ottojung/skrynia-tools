use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct State {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub config: super::config::Config,
    pub client: reqwest::Client,
    pub permits: Arc<Semaphore>,
}

impl State {
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_millis(self.inner.config.acquire_timeout_ms)
    }
    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.inner.config.request_timeout_ms)
    }
    pub fn upload_idle(&self) -> Duration {
        Duration::from_millis(self.inner.config.upload_idle_timeout_ms)
    }
    pub fn upstream_read_idle(&self) -> Duration {
        Duration::from_millis(self.inner.config.upstream_read_timeout_ms)
    }
}
