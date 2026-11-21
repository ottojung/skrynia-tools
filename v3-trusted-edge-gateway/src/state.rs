use std::sync::Arc;

#[derive(Clone)]
pub struct State {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub config: super::config::Config,
    pub client: reqwest::Client,
}
