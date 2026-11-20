use super::State;
use axum::http::{HeaderMap, StatusCode, header::HOST};
pub fn verify(state: &State, headers: &HeaderMap) -> Result<(), StatusCode> {
    let given = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    if given != state.bind.to_string() {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}
