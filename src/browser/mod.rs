//! Browser-facing, asynchronous Wasm bindings for Skrynia.
mod fetch;
mod log_append;
mod log_read;
mod store;
mod types;
pub(super) use super::wire;
pub use types::BrowserClient;

use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(js_name = sha256Hex)]
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn error(e: anyhow::Error) -> JsValue {
    JsValue::from_str(&e.to_string())
}
