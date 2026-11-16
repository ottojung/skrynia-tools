use super::{BrowserClient, wire};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl BrowserClient {
    #[wasm_bindgen(js_name = appendLog)]
    pub async fn append_log(
        &self,
        prefix: String,
        operation_id: String,
        data: &[u8],
    ) -> Result<u64, JsValue> {
        if !wire::valid_prefix(&prefix) {
            return Err(JsValue::from_str("invalid log prefix"));
        }
        let encoded =
            wire::encode(&operation_id, data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        if self.get(wire::key(&prefix, 0)).await?.map(|x| x.to_vec())
            != Some(b"skrynia-log-genesis-v1".to_vec())
        {
            return Err(JsValue::from_str("log not initialized"));
        }
        for n in 1..=1_000_000_u64 {
            let key = wire::key(&prefix, n);
            if let Some(old) = self.get(key.clone()).await? {
                if wire::observed(&old.to_vec(), &encoded, &operation_id)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?
                {
                    return Ok(n);
                }
                continue;
            }
            match self.create(key.clone(), &encoded).await {
                Ok(true) => return Ok(n),
                Ok(false) => match self.observe_log_slot(&key, &encoded, &operation_id).await? {
                    Some(true) => return Ok(n),
                    Some(false) => continue,
                    None => return Err(JsValue::from_str("occupied log slot disappeared")),
                },
                Err(error) => match self.observe_log_slot(&key, &encoded, &operation_id).await {
                    Ok(Some(true)) => return Ok(n),
                    Ok(Some(false)) => continue,
                    Ok(None) => return Err(error),
                    Err(check) => return Err(check),
                },
            }
        }
        Err(JsValue::from_str("log exceeds bounded scan"))
    }
}
