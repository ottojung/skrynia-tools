pub(super) use super::types::BrowserClient;
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
mod bytes;
mod policy;

impl BrowserClient {
    pub(super) async fn fetch(
        &self,
        method: &str,
        url: &str,
        body: Option<&[u8]>,
        authorize: bool,
        digest: Option<&str>,
    ) -> Result<Response, JsValue> {
        let init = RequestInit::new();
        init.set_method(method);
        policy::apply(&init, method);
        if let Some(bytes) = body {
            let data = js_sys::Uint8Array::from(bytes);
            init.set_body(&data.into());
        }
        let request = Request::new_with_str_and_init(url, &init)?;
        if authorize {
            let token = self
                .token
                .as_deref()
                .ok_or_else(|| JsValue::from_str("missing user token"))?;
            request
                .headers()
                .set("Authorization", &format!("Bearer {token}"))?;
        }
        if let Some(value) = digest {
            request.headers().set("X-Skrynia-If-SHA256", value)?;
        }
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        response
            .dyn_into()
            .map_err(|_| JsValue::from_str("invalid fetch response"))
    }
}
