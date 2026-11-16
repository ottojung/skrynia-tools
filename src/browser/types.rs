use super::wire;
use url::Url;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct BrowserClient {
    pub(super) base: Url,
    pub(super) token: Option<String>,
}
#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str, user_token: Option<String>) -> Result<BrowserClient, JsValue> {
        let mut base = Url::parse(url).map_err(|e| JsValue::from_str(&e.to_string()))?;
        if !matches!(base.scheme(), "http" | "https")
            || base.cannot_be_a_base()
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
        {
            return Err(JsValue::from_str("invalid Skrynia base URL"));
        }
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }
        Ok(BrowserClient {
            base,
            token: user_token,
        })
    }
}
impl BrowserClient {
    pub(super) fn key_url(&self, key: &str) -> Result<String, JsValue> {
        if !wire::valid_key(key) {
            return Err(JsValue::from_str("invalid Skrynia object key"));
        }
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| JsValue::from_str("invalid base URL"))?
            .extend(["store", key]);
        Ok(url.to_string())
    }
}
