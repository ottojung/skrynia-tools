use super::{BrowserClient, wire};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = initializeLog)]
    pub async fn initialize_log(&self, prefix: String) -> Result<(), JsValue> {
        if !wire::valid_prefix(&prefix) {
            return Err(JsValue::from_str("invalid log prefix"));
        }
        let key = wire::key(&prefix, 0);
        if self.create(key.clone(), b"skrynia-log-genesis-v1").await? {
            return Ok(());
        }
        let old = self
            .get(key)
            .await?
            .ok_or_else(|| JsValue::from_str("missing genesis"))?;
        if old.to_vec() != b"skrynia-log-genesis-v1" {
            return Err(JsValue::from_str("foreign log genesis"));
        }
        Ok(())
    }
}
