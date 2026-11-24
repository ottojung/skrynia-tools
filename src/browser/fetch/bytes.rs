use super::native;
use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{ReadableStreamDefaultReader, Response};

pub(super) async fn bounded(response: &Response) -> Result<Vec<u8>, JsValue> {
    let reader = response
        .body()
        .ok_or_else(|| JsValue::from_str("missing response body"))?
        .get_reader()
        .dyn_into::<ReadableStreamDefaultReader>()?;
    let mut bytes = Vec::new();
    loop {
        let item = JsFuture::from(reader.read()).await?;
        let done = Reflect::get(&item, &JsValue::from_str("done"))?.as_bool();
        if done == Some(true) {
            return Ok(bytes);
        }
        if done != Some(false) {
            let _ = reader.cancel();
            return Err(JsValue::from_str("invalid response stream"));
        }
        let value = Reflect::get(&item, &JsValue::from_str("value"))?.dyn_into::<Uint8Array>()?;
        let len = value.length() as usize;
        if len > native::MAX_FRAME - bytes.len() {
            let _ = reader.cancel();
            return Err(JsValue::from_str("response too large; outcome unknown"));
        }
        let start = bytes.len();
        bytes.resize(start + len, 0);
        value.copy_to(&mut bytes[start..]);
    }
}
