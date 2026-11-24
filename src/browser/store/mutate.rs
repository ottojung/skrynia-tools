use super::{BrowserClient, error, native};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = create)]
    pub async fn create(&self, key: String, bytes: &[u8]) -> Result<bool, JsValue> {
        let cap = self.caps.writer().map_err(error)?;
        let frame = native::put(cap, &key, None, bytes).map_err(error)?;
        native::written(&self.post(frame).await?, bytes).map_err(error)
    }
    #[wasm_bindgen(js_name = replace)]
    pub async fn replace(
        &self,
        key: String,
        bytes: &[u8],
        digest: String,
    ) -> Result<bool, JsValue> {
        let cap = self.caps.writer().map_err(error)?;
        let hash = native::parse_cap(&digest).map_err(error)?;
        let frame = native::put(cap, &key, Some(&hash), bytes).map_err(error)?;
        native::written(&self.post(frame).await?, bytes).map_err(error)
    }
    #[wasm_bindgen(js_name = delete)]
    pub async fn delete(&self, key: String, digest: String) -> Result<bool, JsValue> {
        let cap = self.caps.writer().map_err(error)?;
        let hash = native::parse_cap(&digest).map_err(error)?;
        let frame = native::delete(cap, &key, &hash).map_err(error)?;
        native::deleted(&self.post(frame).await?).map_err(error)
    }
}
