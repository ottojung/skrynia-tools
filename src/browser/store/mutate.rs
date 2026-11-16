use super::BrowserClient;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = create)]
    pub async fn create(&self, key: String, bytes: &[u8]) -> Result<bool, JsValue> {
        let url = self.key_url(&key)?;
        let response = self.fetch("POST", &url, Some(bytes), true, None).await?;
        if response.status() == 409 {
            return Ok(false);
        }
        Self::accept_exact(&response, 201)?;
        Ok(true)
    }
    #[wasm_bindgen(js_name = replace)]
    pub async fn replace(
        &self,
        key: String,
        bytes: &[u8],
        digest: Option<String>,
    ) -> Result<bool, JsValue> {
        let url = self.key_url(&key)?;
        let response = self
            .fetch("PUT", &url, Some(bytes), true, digest.as_deref())
            .await?;
        if response.status() == 412 {
            return Ok(false);
        }
        Self::accept_exact(&response, 200)?;
        Ok(true)
    }
    #[wasm_bindgen(js_name = delete)]
    pub async fn delete(&self, key: String, digest: Option<String>) -> Result<bool, JsValue> {
        let url = self.key_url(&key)?;
        let response = self
            .fetch("DELETE", &url, None, true, digest.as_deref())
            .await?;
        if response.status() == 412 {
            return Ok(false);
        }
        Self::accept_exact(&response, 200)?;
        Ok(true)
    }
}
