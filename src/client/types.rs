use super::native::Capabilities;
use anyhow::{Result, bail};
use reqwest::{Url, blocking::Client};

#[derive(Clone)]
pub struct SkryniaClient {
    pub(super) endpoint: Url,
    pub(super) http: Client,
    pub(super) edge: String,
    pub(super) caps: Capabilities,
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
    pub fn new(url: &str, edge: &str, read: &str, write: Option<&str>) -> Result<Self> {
        let endpoint = Url::parse(url).map_err(|_| anyhow::anyhow!("invalid endpoint"))?;
        let loopback = matches!(
            endpoint.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]")
        );
        if !(endpoint.scheme() == "https" || (loopback && endpoint.scheme() == "http"))
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || edge.is_empty()
            || !edge.bytes().all(|c| c.is_ascii_graphic())
        {
            bail!("invalid endpoint or edge capability");
        }
        Ok(Self {
            endpoint,
            edge: edge.to_owned(),
            caps: Capabilities::new(read, write)?,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        })
    }
}
