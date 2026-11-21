use crate::{State, app, config};
use axum::http::HeaderMap;

pub struct AuthenticatedApp {
    pub _name: String,
}

pub fn authenticate(
    state: &State,
    headers: &HeaderMap,
) -> Result<AuthenticatedApp, axum::http::StatusCode> {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .ok_or(axum::http::StatusCode::FORBIDDEN)?;
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or(axum::http::StatusCode::FORBIDDEN)?;
    let cap = headers
        .get("x-skrynia-edge-capability")
        .and_then(|v| v.to_str().ok())
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Deny wrong hosts / wrong origin / sibling: must resolve to same configured app.
    let app_cfg = state
        .inner
        .config
        .find_app(host, origin)
        .ok_or(axum::http::StatusCode::FORBIDDEN)?;

    // Separate edge client/session/capability authentication.
    if !app::capability_matches(app_cfg, cap) {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    // Opaque transport: require binary content type.
    match headers
        .get(axum::http::header::CONTENT_TYPE)
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
        _ => return Err(axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE),
    }

    // Normalize once more implicitly via match; reject if normalization would differ in meaning (handled in find_app).
    let _ = config::normalize_host(host);
    let _ = config::normalize_origin(origin);

    Ok(AuthenticatedApp {
        _name: app_cfg.name.clone(),
    })
}
