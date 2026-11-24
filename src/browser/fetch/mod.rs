use super::{types::BrowserClient, wire::native};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestCache, RequestInit, RequestRedirect, Response};

mod bytes;

impl BrowserClient {
    pub(super) async fn post(&self, frame: Vec<u8>) -> Result<Vec<u8>, JsValue> {
        let init = RequestInit::new();
        init.set_method("POST");
        init.set_redirect(RequestRedirect::Error);
        init.set_cache(RequestCache::NoStore);
        init.set_body(&js_sys::Uint8Array::from(frame.as_slice()).into());
        let request = Request::new_with_str_and_init(self.endpoint.as_str(), &init)?;
        request.headers().set(native::EDGE_HEADER, &self.edge)?;
        request
            .headers()
            .set("Content-Type", "application/octet-stream")?;
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
        let response: Response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| JsValue::from_str("transport failed; outcome unknown"))?
            .dyn_into()
            .map_err(|_| JsValue::from_str("invalid response"))?;
        if response.status() != 200 {
            return Err(JsValue::from_str("unexpected HTTP status; outcome unknown"));
        }
        bytes::bounded(&response).await
    }
}
