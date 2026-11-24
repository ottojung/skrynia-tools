use super::wire::native::Capabilities;
use url::Url;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct BrowserClient {
    pub(super) endpoint: Url,
    pub(super) edge: String,
    pub(super) caps: Capabilities,
}
#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        url: &str,
        edge: String,
        read: String,
        write: Option<String>,
    ) -> Result<BrowserClient, JsValue> {
        let endpoint = Url::parse(url).map_err(|_| JsValue::from_str("invalid endpoint"))?;
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
            return Err(JsValue::from_str("invalid endpoint or edge capability"));
        }
        let caps = Capabilities::new(&read, write.as_deref()).map_err(super::error)?;
        Ok(Self {
            endpoint,
            edge,
            caps,
        })
    }
}
