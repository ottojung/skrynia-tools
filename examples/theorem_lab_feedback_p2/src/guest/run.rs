use super::cas::cas_put;
use super::input::{parse, parse_hex32};
use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecError {
    BadInput,
    Rejected(&'static str),
    Busy,
    Io,
}
pub fn submit_json<F>(input: &[u8], mut invoke: F) -> Result<Vec<u8>, ExecError>
where
    F: FnMut(&[u8], &mut [u8]) -> i32,
{
    let inp = parse(input).map_err(|_| ExecError::BadInput)?;
    let iid = parse_hex32(&inp.invoke_id).map_err(|_| ExecError::BadInput)?;
    let (_state, created, retries) =
        cas_put(&iid, &inp.entry, inp.fragments, inp.updated_at, &mut invoke)?;
    let out = serde_json::json!({
        "ok": true, "v": 1, "op": "submit",
        "id": inp.entry.id, "created": created, "retries": retries
    });
    serde_json::to_vec(&out).map_err(|_| ExecError::Io)
}
