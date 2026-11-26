#![cfg(target_arch = "wasm32")]
use super::run::ExecError;
use super::run::submit_json;
use crate::{invoke_raw, read_all, write_all};
use alloc::vec::Vec;

fn err(code: &str) -> Vec<u8> {
    let v = serde_json::json!({"ok": false, "error": code});
    serde_json::to_vec(&v).unwrap_or_else(|_| b"{\"ok\":false,\"error\":\"io\"}".to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let input = read_all(0);
    let bytes = match submit_json(&input, |req, out| invoke_raw(req, out)) {
        Ok(b) => b,
        Err(ExecError::BadInput) => err("bad_input"),
        Err(ExecError::Rejected(c)) => err(c),
        Err(ExecError::Busy) => err("busy"),
        Err(ExecError::Io) => err("io"),
    };
    write_all(0, &bytes);
}
