use super::{BrowserClient, wire};
use wasm_bindgen::prelude::*;

impl BrowserClient {
    pub(super) async fn observe_log_slot(
        &self,
        key: &str,
        expected: &[u8],
        operation_id: &str,
    ) -> Result<Option<bool>, JsValue> {
        self.get(key.to_owned())
            .await?
            .map(|found| {
                wire::observed(&found.to_vec(), expected, operation_id)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            })
            .transpose()
    }
}
