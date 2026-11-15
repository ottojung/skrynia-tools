use super::{types::BrowserClient, wire};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = listPage)]
    pub async fn list_page(
        &self,
        matcher: String,
        after: Option<String>,
    ) -> Result<String, JsValue> {
        if !wire::valid_matcher(&matcher) {
            return Err(JsValue::from_str("invalid matcher"));
        }
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| JsValue::from_str("invalid base URL"))?
            .push("store");
        url.query_pairs_mut()
            .append_pair("locations", &matcher)
            .append_pair("limit", "1000");
        if let Some(after) = after {
            url.query_pairs_mut().append_pair("after", &after);
        }
        let response = self.fetch("GET", url.as_str(), None, true, None).await?;
        Self::accept(&response)?;
        String::from_utf8(Self::bytes(&response).await?)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
