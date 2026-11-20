use super::{State, headers, host, token, upstream};
use axum::{
    body::Bytes,
    extract::{OriginalUri, State as Extract},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
pub async fn handle(
    Extract(state): Extract<std::sync::Arc<State>>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    match forward(&state, method, uri, headers, body).await {
        Ok(r) => r,
        Err(e) => e.into_response(),
    }
}
async fn forward(
    state: &State,
    method: Method,
    uri: axum::http::Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, StatusCode> {
    // Reject unexpected Host before even opening the private bearer file.
    host::verify(state, &headers)?;
    let token = token::load(&state.token_path).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let r = skrynia_tools::wire::v2compat::Request {
        method: method.to_string(),
        path: uri.path().into(),
        query: uri.query().map(str::to_string),
        headers: headers::from_request(&headers).map_err(|_| StatusCode::BAD_REQUEST)?,
        body: body.to_vec(),
    };
    let request = serde_json::to_vec(&r).map_err(|_| StatusCode::BAD_REQUEST)?;
    if request.len() > 8 * 1024 * 1024 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let mut payload = Vec::with_capacity(request.len() + 33);
    payload.push(1);
    payload.extend_from_slice(&token);
    payload.extend_from_slice(&request);
    let answer = upstream::post(state, payload).await?;
    let response = serde_json::from_slice(&answer).map_err(|_| StatusCode::BAD_GATEWAY)?;
    headers::into_response(response).map_err(|_| StatusCode::BAD_GATEWAY)
}
