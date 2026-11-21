use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::{Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};

pub async fn reject(_state: AxumState<crate::state::State>, req: Request<Body>) -> Response {
    if req.method() == Method::POST {
        return StatusCode::NOT_FOUND.into_response();
    }
    StatusCode::METHOD_NOT_ALLOWED.into_response()
}
