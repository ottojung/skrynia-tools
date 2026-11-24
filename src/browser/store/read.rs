use super::{BrowserClient, error, native};
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = get)]
    pub async fn get(&self, key: String) -> Result<Option<js_sys::Uint8Array>, JsValue> {
        let frame = native::get(&self.caps.read, &key).map_err(error)?;
        Ok(native::read(&self.post(frame).await?)
            .map_err(error)?
            .map(|value| js_sys::Uint8Array::from(value.as_slice())))
    }
}
