use crate::{State, auth, token};
use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use futures_util::stream::{self, StreamExt};
use http_body_util::BodyExt;

pub async fn handle(AxumState(state): AxumState<State>, req: Request<Body>) -> Response {
    if req.method() != Method::POST {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let (parts, body) = req.into_parts();
    // Authenticate before forwarding anything; operator token loaded only after positive auth.
    let _app = match auth::authenticate(&state, &parts.headers) {
        Ok(app) => app,
        Err(status) => return status.into_response(),
    };

    let max = state.inner.config.max_body_bytes;
    if let Some(len) = parts
        .headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        if len > max {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
    }

    let operator = match token::load_operator_token(&state.inner.config.operator_token_file) {
        Ok(t) => t,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    // Stream request body with bound enforcement.
    let limited = LimitedBody {
        inner: body,
        remaining: max,
        overflow: false,
    };
    let data_stream = limited
        .into_data_stream()
        .map(|r| r.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "body read error")));
    let prefix = Bytes::from([vec![0x01u8], operator.to_vec()].concat());
    let upstream_body = reqwest::Body::wrap_stream(
        stream::once(async move { Ok::<Bytes, std::io::Error>(prefix) }).chain(data_stream),
    );

    let upstream_url = match state.inner.config.upstream.as_str().parse::<reqwest::Url>() {
        Ok(u) => u,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let mut req_builder = state
        .inner
        .client
        .post(upstream_url)
        .header("content-type", "application/octet-stream");
    if let Some(ct) = parts.headers.get(axum::http::header::CONTENT_TYPE) {
        if let Ok(v) = ct.to_str() {
            req_builder = req_builder.header("content-type", v);
        }
    }
    // Do not forward edge credentials or hop-by-hop/secret headers to kernel.
    req_builder = req_builder.body(upstream_body);

    let upstream = match req_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            // Distinguish oversized/aborted client body from upstream issues is non-trivial once streaming started;
            // map to 502/400 conservatively. (Detailed cause not exposed.)
            let _ = e;
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let status = upstream.status();
    let mut resp_headers = HeaderMap::new();
    // Forward content-type only (safe) and no-store; do not leak upstream secrets.
    if let Some(ct) = upstream.headers().get(reqwest::header::CONTENT_TYPE) {
        resp_headers.insert(axum::http::header::CONTENT_TYPE, ct.clone());
    }
    resp_headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::header::HeaderValue::from_static("no-store"),
    );

    let byte_stream = upstream
        .bytes_stream()
        .scan(0usize, move |seen, item| {
            let max = max;
            let out = match item {
                Ok(bytes) => match seen.checked_add(bytes.len()) {
                    Some(v) if v <= max => {
                        *seen = v;
                        Some(Ok(bytes))
                    }
                    _ => Some(Err(())),
                },
                Err(_) => Some(Err(())),
            };
            futures_util::future::ready(out)
        })
        .map(|r| r.map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "upstream error")));

    let body = Body::from_stream(byte_stream);
    let mut response = Response::new(body);
    *response.status_mut() =
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    *response.headers_mut() = resp_headers;
    response
}

pub async fn reject(_state: AxumState<State>, req: Request<Body>) -> Response {
    if req.method() == Method::POST {
        // Only exact /_skrynia/ path is served; avoid leaking info.
        return StatusCode::NOT_FOUND.into_response();
    }
    StatusCode::METHOD_NOT_ALLOWED.into_response()
}

struct LimitedBody {
    inner: Body,
    remaining: usize,
    overflow: bool,
}

impl http_body::Body for LimitedBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        if self.overflow {
            return std::task::Poll::Ready(Some(Err(axum::Error::new("body too large"))));
        }
        let this = &mut *self;
        match std::pin::Pin::new(&mut this.inner).poll_frame(cx) {
            std::task::Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    if data.len() > this.remaining {
                        this.overflow = true;
                        return std::task::Poll::Ready(Some(Err(axum::Error::new(
                            "body too large",
                        ))));
                    }
                    this.remaining -= data.len();
                }
                std::task::Poll::Ready(Some(Ok(frame)))
            }
            other => other,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
