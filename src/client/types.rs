use anyhow::{Result, bail};
use reqwest::{Url, blocking::Client};

#[derive(Clone)]
pub struct SkryniaClient {
    pub(super) base: Url,
    pub(super) http: Client,
    pub(super) token: Option<String>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateResult {
    Created,
    Exists,
}
pub trait Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn create(&self, key: &str, data: &[u8]) -> Result<CreateResult>;
}
impl SkryniaClient {
    pub fn new(url: &str) -> Result<Self> {
        let mut base = Url::parse(url)?;
        if !matches!(base.scheme(), "http" | "https")
            || base.query().is_some()
            || base.fragment().is_some()
            || base.cannot_be_a_base()
            || !base.username().is_empty()
            || base.password().is_some()
        {
            bail!("invalid Skrynia base URL");
        }
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }
        Ok(Self {
            base,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            token: None,
        })
    }
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }
}
