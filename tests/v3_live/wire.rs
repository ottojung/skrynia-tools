use skrynia_native_service::{
    wdec::decode_res,
    wenc::encode_req,
    wire::{Request, STATUS_OK},
};
pub fn call(url: &str, request: Request) -> Result<Vec<u8>, String> {
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
    let reply = decode_res(&payload).map_err(|e| format!("wire {e:?}"))?;
    if reply.status != STATUS_OK {
        return Err(format!(
            "application {}, code {:?}",
            reply.status, reply.body
        ));
    }
    Ok(reply.body)
}
pub fn invoke(url: &str, invoke_id: &[u8]) -> Result<Vec<u8>, String> {
    call(
        url,
        Request::Invoke {
            invoke_id: invoke_id.to_vec(),
            input: b"test".to_vec(),
        },
    )
}
