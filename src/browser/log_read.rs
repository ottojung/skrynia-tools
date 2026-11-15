use super::types::BrowserClient;
use super::wire;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = readLog)]
    pub async fn read_log(
        &self,
        prefix: String,
        start: u64,
        limit: u32,
    ) -> Result<String, JsValue> {
        if !wire::valid_prefix(&prefix) {
            return Err(JsValue::from_str("invalid log prefix"));
        }
        let mut out = Vec::new();
        for n in start.max(1)..start.max(1).saturating_add(limit.min(1000) as u64) {
            let Some(value) = self.get(wire::key(&prefix, n)).await? else {
                break;
            };
            let bytes = value.to_vec();
            let (id, payload) =
                wire::decode(&bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
            out.push(serde_json::json!({"position":n,"operation_id":id,
                "payload_hex":hex::encode(payload)}));
        }
        serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
