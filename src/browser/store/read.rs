use super::BrowserClient;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = get)]
    pub async fn get(&self, key: String) -> Result<Option<js_sys::Uint8Array>, JsValue> {
        let url = self.key_url(&key)?;
        let response = self.fetch("GET", &url, None, false, None).await?;
        if response.status() == 404 {
            return Ok(None);
        }
        Self::accept(&response)?;
        Ok(Some(js_sys::Uint8Array::from(
            Self::bytes(&response).await?.as_slice(),
        )))
    }
}
