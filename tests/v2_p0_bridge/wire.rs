use sha2::{Digest, Sha256};
use skrynia_program0::{
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, STATUS_ERR, STATUS_OK},
};
pub fn sha256(b: &[u8]) -> [u8; 32] {
    Sha256::new().chain_update(b).finalize().into()
}
pub fn ok(url: &str, request: Request) -> Result<(), String> {
    let r = call(url, request)?;
    if r.status == STATUS_OK {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&r.body).into())
    }
}
pub fn err(url: &str, request: Request) -> Result<String, String> {
    let r = call(url, request)?;
    if r.status == STATUS_ERR {
        Ok(String::from_utf8_lossy(&r.body).into_owned())
    } else {
        Err("expected error".into())
    }
}
fn call(url: &str, request: Request) -> Result<skrynia_program0::wire::Response, String> {
    let res = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?
        .post(url)
        .header("content-type", "application/octet-stream")
        .body(encode_req(&request))
        .send()
        .map_err(|e| e.to_string())?;
    if res.status() != 200 {
        return Err(format!("HTTP {}", res.status()));
    }
    let payload = res.bytes().map_err(|e| e.to_string())?;
    decode_res(&payload).map_err(|e| format!("wire {e:?}"))
}
pub fn mark_uncertain(dir: &std::path::Path) {
    std::fs::create_dir_all(dir.join("v2auth")).unwrap();
    std::fs::write(dir.join("v2auth/uncertain"), b"1").unwrap();
}
