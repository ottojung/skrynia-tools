use axum::http::{HeaderMap, StatusCode, header};

pub struct AuthenticatedApp {
    pub _name: String,
}

pub fn authenticate(
    state: &crate::state::State,
    headers: &HeaderMap,
) -> Result<AuthenticatedApp, StatusCode> {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    let cap = headers
        .get("x-skrynia-edge-capability")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let app_cfg = state
        .inner
        .config
        .find_app(host, origin)
        .ok_or(StatusCode::FORBIDDEN)?;
    if !crate::app::capability_matches(app_cfg, cap) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    match headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase()
        })
        .as_deref()
    {
        Some("application/octet-stream") => {}
        _ => return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE),
    }
    Ok(AuthenticatedApp {
        _name: app_cfg.name.clone(),
    })
}
