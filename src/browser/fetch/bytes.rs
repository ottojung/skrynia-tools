use super::BrowserClient;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

impl BrowserClient {
    pub(super) fn accept(response: &Response) -> Result<(), JsValue> {
        if (200..300).contains(&response.status()) {
            return Ok(());
        }
        Err(JsValue::from_str(&format!(
            "Skrynia HTTP {}",
            response.status()
        )))
    }
    pub(super) async fn bytes(response: &Response) -> Result<Vec<u8>, JsValue> {
        let buffer = JsFuture::from(response.array_buffer()?).await?;
        Ok(js_sys::Uint8Array::new(&buffer).to_vec())
    }
}
