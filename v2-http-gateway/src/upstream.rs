use super::State;
use axum::http::StatusCode;
pub async fn post(state: &State, payload: Vec<u8>) -> Result<Vec<u8>, StatusCode> {
    let mut response = state
        .client
        .post(state.endpoint.clone())
        .header("content-type", "application/octet-stream")
        .body(payload)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let mut out = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
    {
        if out
            .len()
            .checked_add(chunk.len())
            .is_none_or(|n| n > 10 * 1024 * 1024)
        {
            return Err(StatusCode::BAD_GATEWAY);
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}
