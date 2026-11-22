//! Fail-closed precondition checks before any socket is opened.
use super::PushRequest;

pub fn valid(req: &PushRequest) -> bool {
    req.port == 443
        && !req.host.is_empty()
        && req.host == req.host.to_ascii_lowercase()
        && req.path.starts_with('/')
        && !req
            .path
            .bytes()
            .any(|b| b < 0x20 || b == 0x7f || b == b'#' || b == b'\\')
}
